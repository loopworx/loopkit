//! `gen_coq` — Generate the Coq formalization of the skill loop state machine
//! directly from the parsed handoff graph.
//!
//! Auto-discovers states, transitions, entry points, and terminal states
//! from the skill files. Emits `Generated.v` and an OCaml checker.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::process::ExitCode;

use skill_loop_verifier::config::load_config;
use skill_loop_verifier::types::Repo;
use skill_loop_verifier::types::build_adjacency;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return ExitCode::SUCCESS;
    }

    let (root, skills_dir, check, output_dir) = parse_args(&args);

    let config = load_config(&root);
    let skills_dir_name = skills_dir.unwrap_or_else(|| config.skills_dir.clone());

    let repo = match Repo::from_root(root.clone(), &skills_dir_name) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gen_coq: failed to load repo: {e}");
            return ExitCode::FAILURE;
        }
    };

    let generated = emit_generated(&repo);
    let checker = emit_checker_ml(&repo);

    let proof_dir = output_dir.unwrap_or_else(|| root.join("proofs"));
    let out_generated = proof_dir.join("Generated.v");
    let out_checker = proof_dir.join("extraction/checker.ml");

    if check {
        match std::fs::read_to_string(&out_generated) {
            Ok(existing) if existing == generated => {}
            Ok(_) => {
                eprintln!("gen_coq: Generated.v is STALE — regenerate with `gen-coq`");
                return ExitCode::FAILURE;
            }
            Err(_) => {
                eprintln!("gen_coq: Generated.v is MISSING — generate with `gen-coq`");
                return ExitCode::FAILURE;
            }
        }
        match std::fs::read_to_string(&out_checker) {
            Ok(existing) if existing == checker => {}
            Ok(_) => {
                eprintln!("gen_coq: checker.ml is STALE — regenerate with `gen-coq`");
                return ExitCode::FAILURE;
            }
            Err(_) => {
                eprintln!("gen_coq: checker.ml is MISSING — generate with `gen-coq`");
                return ExitCode::FAILURE;
            }
        }
        println!("gen_coq: Generated.v and checker.ml are up to date.");
        return ExitCode::SUCCESS;
    }

    // Create directories
    let _ = std::fs::create_dir_all(&proof_dir);
    let _ = std::fs::create_dir_all(proof_dir.join("extraction"));

    if let Err(e) = std::fs::write(&out_generated, &generated) {
        eprintln!("gen_coq: failed to write {}: {e}", out_generated.display());
        return ExitCode::FAILURE;
    }
    println!("gen_coq: wrote {}", out_generated.display());
    if let Err(e) = std::fs::write(&out_checker, &checker) {
        eprintln!("gen_coq: failed to write {}: {e}", out_checker.display());
        return ExitCode::FAILURE;
    }
    println!("gen_coq: wrote {}", out_checker.display());
    ExitCode::SUCCESS
}

fn parse_args(args: &[String]) -> (PathBuf, Option<String>, bool, Option<PathBuf>) {
    let mut root: Option<PathBuf> = None;
    let mut skills_dir: Option<String> = None;
    let mut check = false;
    let mut output_dir: Option<PathBuf> = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => root = iter.next().map(PathBuf::from),
            "--skills-dir" => skills_dir = iter.next().cloned(),
            "--check" => check = true,
            "--output-dir" => output_dir = iter.next().map(PathBuf::from),
            _ => {}
        }
    }
    let root = root.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    (root, skills_dir, check, output_dir)
}

fn print_help() {
    eprintln!("Usage: gen_coq [--root <path>] [--skills-dir <dir>] [--output-dir <dir>] [--check]");
}

// ── Graph helpers ────────────────────────────────────────────────────

fn shortest_path_to_any(
    start: &str,
    targets: &HashSet<String>,
    adj: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    if targets.contains(start) {
        return Some(vec![start.to_string()]);
    }
    let mut prev: HashMap<String, String> = HashMap::new();
    let mut visited = HashSet::new();
    visited.insert(start.to_string());
    let mut queue: VecDeque<String> = VecDeque::new();
    queue.push_back(start.to_string());
    while let Some(node) = queue.pop_front() {
        let Some(neighbors) = adj.get(&node) else {
            continue;
        };
        for next in neighbors {
            if visited.contains(next) {
                continue;
            }
            visited.insert(next.clone());
            prev.insert(next.clone(), node.clone());
            if targets.contains(next) {
                let mut path = vec![next.clone()];
                let mut cursor = next.clone();
                while let Some(p) = prev.get(&cursor) {
                    path.push(p.clone());
                    cursor = p.clone();
                }
                path.reverse();
                return Some(path);
            }
            queue.push_back(next.clone());
        }
    }
    None
}

fn ctor_id(state: &str) -> String {
    format!("S_{}", state.replace('-', "_"))
}

fn trans_ctor_id(from: &str, to: &str) -> String {
    format!("T_{}_{}", from.replace('-', "_"), to.replace('-', "_"))
}

fn coq_list(items: &[String]) -> String {
    if items.is_empty() {
        "[]".to_string()
    } else {
        format!("[ {} ]", items.join("; "))
    }
}

// ── Generation ───────────────────────────────────────────────────────

fn emit_generated(repo: &Repo) -> String {
    let graph = &repo.handoff_graph;

    // Collect all states sorted
    let mut all_states: Vec<String> = graph.nodes.iter().map(|n| n.name.clone()).collect();
    all_states.sort();

    // Terminal states
    let terminal_set: HashSet<String> = all_states
        .iter()
        .filter(|s| graph.nodes.iter().any(|n| n.name == **s && n.is_terminal))
        .cloned()
        .collect();

    // Entry points
    let entry_set: HashSet<String> = graph
        .entry_points
        .iter()
        .map(|s| s.name.clone())
        .collect();
    let mut entry_points: Vec<String> = entry_set.iter().cloned().collect();
    entry_points.sort();

    // Entry points
    let mut entry_points: Vec<String> = entry_set.iter().cloned().collect();
    entry_points.sort();

    let canon_idx: HashMap<String, usize> = all_states
        .iter()
        .enumerate()
        .map(|(i, s): (usize, &String)| (s.clone(), i))
        .collect();

    // Unique edges
    let mut unique_edges: Vec<(String, String)> = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for t in &graph.edges {
        let pair = (t.from.clone(), t.to.clone());
        if t.from != t.to && seen.insert(pair.clone()) {
            unique_edges.push(pair);
        }
    }
    unique_edges.sort();

    let adj = build_adjacency(&graph.edges);

    // Rank: reverse of canonical order (entry states have larger rank)
    fn rank_of(state: &str, canonical: &[String], idx: &HashMap<String, usize>) -> u32 {
        idx.get(state)
            .map(|&i| (canonical.len() - 1 - i) as u32)
            .unwrap_or(0)
    }

    let mut succ_by_src: HashMap<&String, Vec<&String>> = HashMap::new();
    for e in &unique_edges {
        succ_by_src.entry(&e.0).or_default().push(&e.1);
    }

    let mut out = String::new();
    out.push_str("(** * Generated Skill Loop State Machine\n\n");
    out.push_str("    DO NOT EDIT BY HAND. Generated by skill-loop-verifier gen-coq\n");
    out.push_str("    from the skill handoff graph.\n*)\n\n");

    out.push_str("Require Import Stdlib.Lists.List.\n");
    out.push_str("Import ListNotations.\n\n");

    // State
    out.push_str("Inductive State : Set :=\n");
    for (i, s) in all_states.iter().enumerate() {
        let sep = if i + 1 == all_states.len() { "." } else { "" };
        out.push_str(&format!("  | {}{}\n", ctor_id(s), sep));
    }
    out.push('\n');
    out.push_str("Hint Constructors State : core.\n\n");

    // terminal_states
    let terminal_ids: Vec<String> = all_states
        .iter()
        .filter(|s| terminal_set.contains(*s))
        .map(|s| ctor_id(s))
        .collect();
    out.push_str(&format!(
        "Definition terminal_states : list State := {}.\n\n",
        coq_list(&terminal_ids)
    ));

    // is_terminal
    out.push_str("Definition is_terminal (s : State) : bool :=\n  match s with\n");
    for s in &all_states {
        let b = if terminal_set.contains(s) { "true" } else { "false" };
        out.push_str(&format!("  | {} => {}\n", ctor_id(s), b));
    }
    out.push_str("  end.\n\n");

    // all_states
    let all_ids: Vec<String> = all_states.iter().map(|s| ctor_id(s)).collect();
    out.push_str(&format!(
        "Definition all_states : list State := {}.\n\n",
        coq_list(&all_ids)
    ));

    // entry_points
    let entry_ids: Vec<String> = entry_points.iter().map(|s| ctor_id(s)).collect();
    out.push_str(&format!(
        "Definition entry_points : list State := {}.\n\n",
        coq_list(&entry_ids)
    ));

    // Transition
    out.push_str("Inductive Transition : State -> State -> Prop :=\n");
    if unique_edges.is_empty() {
        out.push_str("  | T_none : False -> Transition S_done S_done.\n");
    } else {
        for (i, (from, to)) in unique_edges.iter().enumerate() {
            let sep = if i + 1 == unique_edges.len() { "." } else { "" };
            out.push_str(&format!(
                "  | {} : Transition {} {}{}\n",
                trans_ctor_id(from, to),
                ctor_id(from),
                ctor_id(to),
                sep
            ));
        }
    }
    out.push('\n');
    out.push_str("Hint Constructors Transition : core.\n\n");

    // successors
    out.push_str("Definition successors (s : State) : list State :=\n  match s with\n");
    for s in &all_states {
        let dests: Vec<String> = adj
            .get(s)
            .map(|v| v.iter().map(|d| ctor_id(d)).collect())
            .unwrap_or_default();
        out.push_str(&format!("  | {} => {}\n", ctor_id(s), coq_list(&dests)));
    }
    out.push_str("  end.\n\n");

    // Rank
    out.push_str("Definition rank (s : State) : nat :=\n  match s with\n");
    for s in &all_states {
        let r = rank_of(s, &all_states, &canon_idx);
        out.push_str(&format!("  | {} => {}\n", ctor_id(s), r));
    }
    out.push_str("  end.\n\n");

    // Reachability inductives
    out.push_str("Inductive Path : list State -> Prop :=\n");
    out.push_str("  | path_singleton : forall s, Path [s]\n");
    out.push_str("  | path_step : forall s1 s2 ss,\n");
    out.push_str("      Transition s1 s2 -> Path (s2 :: ss) -> Path (s1 :: s2 :: ss).\n\n");
    out.push_str("Inductive Reachable : State -> State -> Prop :=\n");
    out.push_str("  | reach_refl : forall s, Reachable s s\n");
    out.push_str("  | reach_step : forall s m t, Transition s m -> Reachable m t -> Reachable s t.\n\n");
    out.push_str("Hint Constructors Path Reachable : core.\n\n");

    // Reachability witnesses from non-terminal states to some terminal
    let done_candidate = terminal_set
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| "done".to_string());
    let done_target = HashSet::from([done_candidate.clone()]);

    // Witnesses for non-terminal states → terminal
    for s in &all_states {
        if terminal_set.contains(s) {
            continue;
        }
        let lemma = format!("{}_reaches_terminal", s.replace('-', "_"));
        match shortest_path_to_any(s, &done_target, &adj) {
            Some(path) if path.len() >= 2 => {
                out.push_str(&format!(
                    "Lemma {} : Reachable {} {}.\nProof.\n",
                    lemma,
                    ctor_id(s),
                    ctor_id(&done_candidate)
                ));
                for w in path.windows(2) {
                    out.push_str(&format!(
                        "  apply (reach_step _ {}). apply {}.\n",
                        ctor_id(&w[1]),
                        trans_ctor_id(&w[0], &w[1])
                    ));
                }
                out.push_str("  apply reach_refl.\nQed.\n\n");
            }
            _ => {}
        }
    }

    // Covering theorem: non-terminal states reach a terminal
    let non_terminal: Vec<&String> = all_states
        .iter()
        .filter(|s| !terminal_set.contains(*s))
        .collect();
    if !non_terminal.is_empty() {
        out.push_str("(** Covering theorem: every non-terminal state reaches a terminal. *)\n");
        let done_ctor = ctor_id(&done_candidate);
        out.push_str(&format!(
            "Theorem non_terminal_states_reach_done :\n  forall s, is_terminal s = false -> Reachable s {}.\n",
            done_ctor
        ));
        out.push_str("Proof.\n  intros s Hne. destruct s; simpl in Hne;\n    try discriminate.\n");
        for s in &non_terminal {
            let lemma = format!("{}_reaches_terminal", s.replace('-', "_"));
            out.push_str(&format!("  - apply {}.\n", lemma));
        }
        out.push_str("Qed.\n\n");
    }

    // Entry point reachability
    let mut entry_witnesses: Vec<(String, Vec<String>)> = Vec::new();
    for ep in &entry_points {
        if let Some(path) = shortest_path_to_any(ep, &done_target, &adj) {
            entry_witnesses.push((ep.clone(), path));
        }
    }
    if !entry_witnesses.is_empty() {
        out.push_str("Theorem entry_points_reach_done :\n  forall s, In s entry_points -> Reachable s ");
        out.push_str(&ctor_id(&done_candidate));
        out.push_str(".\nProof.\n  intros s H. simpl in H.\n");
        // Build destruct pattern
        let mut binds = String::new();
        for i in 0..entry_points.len() {
            if i == 0 { binds.push_str("[H"); }
            else { binds.push_str(" | [H"); }
        }
        binds.push_str(" | Hf");
        for _ in 0..entry_points.len() { binds.push(']'); }
        out.push_str(&format!("  destruct H as {}.\n", binds));
        for ep in &entry_points {
            let lemma = format!("{}_reaches_terminal", ep.replace('-', "_"));
            out.push_str(&format!("  - subst. apply {}.\n", lemma));
        }
        out.push_str("  - inversion Hf.\nQed.\n\n");
    }

    out.push_str(&format!(
        "\n(** [{}] states, [{}] terminal, [{}] entry, [{}] edges. *)\n",
        all_states.len(),
        terminal_set.len(),
        entry_points.len(),
        unique_edges.len()
    ));
    out
}

fn emit_checker_ml(repo: &Repo) -> String {
    let graph = &repo.handoff_graph;
    let all_states: Vec<String> = graph.nodes.iter().map(|n| n.name.clone()).collect();

    let mut out = String::new();
    out.push_str("(** Runtime checker for skill loop state machine.\n\n");
    out.push_str("    DO NOT EDIT BY HAND. Generated by skill-loop-verifier gen-coq.\n");
    out.push_str("*)\n\n");
    out.push_str("open Forge_state_machine\n\n");

    // string_of_state
    out.push_str("let string_of_state = function\n");
    for s in &all_states {
        out.push_str(&format!("  | {} -> \"{}\"\n", ctor_id(s), s));
    }
    out.push('\n');

    // check_no_dead_ends
    out.push_str("let check_no_dead_ends () =\n");
    out.push_str("  let failures = List.fold_left (fun acc s ->\n");
    out.push_str("    if not (is_terminal s) && successors s = [] then s :: acc else acc\n");
    out.push_str("  ) [] all_states in\n");
    out.push_str("  match failures with\n");
    out.push_str("  | [] -> Printf.printf \"OK: every non-terminal state has at least one successor.\\n\"\n");
    out.push_str("  | _ ->\n");
    out.push_str("      Printf.printf \"FAIL: dead-end non-terminal states: %s\\n\"\n");
    out.push_str("        (String.concat \", \" (List.map string_of_state failures));\n");
    out.push_str("      exit 1\n\n");

    // check_terminals_are_sinks
    out.push_str("let check_terminals_are_sinks () =\n");
    out.push_str("  let failures = List.fold_left (fun acc s ->\n");
    out.push_str("    if is_terminal s && successors s <> [] then s :: acc else acc\n");
    out.push_str("  ) [] terminal_states in\n");
    out.push_str("  match failures with\n");
    out.push_str("  | [] -> Printf.printf \"OK: terminal states have no outgoing edges.\\n\"\n");
    out.push_str("  | _ ->\n");
    out.push_str("      Printf.printf \"FAIL: terminal states with outgoing edges: %s\\n\"\n");
    out.push_str("        (String.concat \", \" (List.map string_of_state failures));\n");
    out.push_str("      exit 1\n\n");

    // BFS reachability
    out.push_str("let max_iterations = ");
    out.push_str(&graph.nodes.len().to_string());
    out.push_str("\n\n");
    out.push_str("let check_reachability target entry =\n");
    out.push_str("  let module S = Set.Make(struct type t = state let compare = compare end) in\n");
    out.push_str("  let rec bfs visited frontier steps =\n");
    out.push_str("    if steps > max_iterations then false\n");
    out.push_str("    else match frontier with\n");
    out.push_str("    | [] -> false\n");
    out.push_str("    | _ ->\n");
    out.push_str("        let frontier' = List.filter (fun s -> not (S.mem s visited)) frontier in\n");
    out.push_str("        if List.exists (fun s -> s = target) frontier' then true\n");
    out.push_str("        else begin\n");
    out.push_str("          let visited' = List.fold_left (fun acc s -> S.add s acc) visited frontier' in\n");
    out.push_str("          let next_frontier = List.concat_map (fun s -> successors s) frontier' in\n");
    out.push_str("          let next_frontier' = List.filter (fun s -> not (S.mem s visited')) next_frontier in\n");
    out.push_str("          bfs visited' next_frontier' (steps + 1)\n");
    out.push_str("        end\n");
    out.push_str("  in bfs S.empty [entry] 0\n\n");

    // check_entry_points
    out.push_str("let check_entry_points () =\n");
    out.push_str("  let failures = List.fold_left (fun acc s ->\n");
    out.push_str("    if not (check_reachability S_done s) then s :: acc else acc\n");
    out.push_str("  ) [] entry_points in\n");
    out.push_str("  match failures with\n");
    out.push_str("  | [] -> Printf.printf \"OK: every entry point can reach done.\\n\"\n");
    out.push_str("  | _ ->\n");
    out.push_str("      Printf.printf \"FAIL: entry points that cannot reach done: %s\\n\"\n");
    out.push_str("        (String.concat \", \" (List.map string_of_state failures));\n");
    out.push_str("      exit 1\n\n");

    out.push_str("let () =\n");
    out.push_str("  check_no_dead_ends ();\n");
    out.push_str("  check_terminals_are_sinks ();\n");
    out.push_str("  check_entry_points ();\n");
    out.push_str("  Printf.printf \"All skill loop checks passed.\\n\"\n");
    out
}
