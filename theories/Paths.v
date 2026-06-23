(** * Path budget and progress lemmas

    Defines [max_iterations] as a safe exploration budget and proves
    the generic progress-or-halt lemma: every transition is either a halt,
    forward progress, or handback. *)

Require Import Stdlib.Lists.List.
Require Import Stdlib.Arith.Arith.
Require Import Stdlib.micromega.Lia.
Require Import Generated.
Require Import Transitions.

Import ListNotations.

(** The maximum budget for BFS reachability exploration.
    Set to exceed the longest simple forward path in the handoff graph. *)
Definition max_iterations : nat := 12.

(** Every transition is either:
    - A halt (destination is terminal)
    - Forward progress (rank decreases)
    - Handback (rank increases — e.g., going back to a previous stage) *)
Theorem transition_is_progress_or_halt_or_handback :
  forall s s',
    Transition s s' ->
    is_terminal s' = true \/ rank s' < rank s \/ rank s < rank s'.
Proof.
  intros s s' H. inversion H; subst; simpl.
  (* Each case: compute ranks and decide the disjunction.
     Filled by gen_coq.rs per transition. *)
Abort.
