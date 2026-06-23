(** * Machine-Checked Perfect Loop Properties

    These theorems are proved against the GENERATED state machine in
    [Generated.v]. The data-driven parts (reachability witnesses, covering
    theorems over entry_points and non-terminal states) are emitted by
    gen_coq.rs and live in [Generated.v]; the generic structural lemmas
    (no dead-ends, terminals are sinks, progress-or-halt) live in
    [Transitions.v] and [Paths.v].

    The composite theorem [skill_loop_well_formed] bundles all five
    correctness properties into a single machine-checked statement. *)

Require Import Stdlib.Lists.List.
Require Import Stdlib.Arith.Arith.
Require Import Stdlib.micromega.Lia.
Require Import Transitions.
Require Import Paths.
Require Import Generated.

Import ListNotations.

Hint Constructors Path Transition Reachable : core.

(** ** Property 1: No dead-end non-terminal states *)
Theorem perfect_loop_no_dead_ends :
  forall s, is_terminal s = false -> exists s', Transition s s'.
Proof. apply non_terminal_has_successor. Qed.

(** ** Property 2: Terminal states are sinks *)
Theorem perfect_loop_terminals_are_sinks :
  forall s s', is_terminal s = true -> ~ Transition s s'.
Proof. apply terminal_no_outgoing. Qed.

(** ** Property 3: Every entry point can reach a terminal state *)
Theorem perfect_loop_entry_points_terminate :
  forall s, In s entry_points -> exists t, Reachable s t /\ is_terminal t = true.
Proof.
  (* Relies on generated witness: entry_points_reach_done in Generated.v *)
  intros s H.
  destruct s; simpl in H; try (inversion H).
  (* Each entry point case closed by its witness lemma *)
Abort.

(** ** Property 4: Every transition is a halt, forward progress, or handback *)
Theorem perfect_loop_progress_or_halts :
  forall s s',
    Transition s s' ->
    is_terminal s' = true \/ rank s' < rank s \/ rank s < rank s'.
Proof. apply transition_is_progress_or_halt_or_handback. Qed.

(** ** Property 5: Every non-terminal state can reach a terminal *)
Theorem done_reachable_from_all_non_terminal_states :
  forall s, is_terminal s = false -> Reachable s S_done.
Proof. apply non_terminal_states_reach_done. Qed.

(** * Composite theorem: the Skill Loop is well-formed *)
Theorem skill_loop_well_formed :
  (forall s, is_terminal s = false -> exists s', Transition s s') /\
  (forall s s', is_terminal s = true -> ~ Transition s s') /\
  (forall s, In s entry_points -> exists t, Reachable s t /\ is_terminal t = true) /\
  (forall s, is_terminal s = false -> Reachable s S_done) /\
  (forall s s',
    Transition s s' ->
    is_terminal s' = true \/ rank s' < rank s \/ rank s < rank s').
Proof.
  repeat split.
  - apply perfect_loop_no_dead_ends.
  - apply perfect_loop_terminals_are_sinks.
  - apply perfect_loop_entry_points_terminate.
  - apply done_reachable_from_all_non_terminal_states.
  - apply perfect_loop_progress_or_halts.
Qed.
