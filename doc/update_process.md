## Basic structures

`sth` table stores any sth received that has a valid signature, whether or not it is consistent, whether it come directly from the log or gossip, etc. Each stored sth has a internal `id`.

## Update process

Whenever the server gets a sth directly from a log, it stores the sth it got into the `sth` table with `checked_consistent_with_latest` set to `false`. Unless the newly gotton sth has a `tree_size` less than or equal to that of the current `latest_sth`, it then proceeds to check consistency with the current `latest_sth` and fetch the new certificates, and only when both is successful will it update `ctlogs`.`latest_sth` to point to the new sth, and at the same time, set `checked_consistent_with_latest` to `true`.

After this process is done, whether `latest_sth` is updated or not, we find all sth in the `sth` table with `tree_size` less than or equal to that of the `latest_sth` and `checked_consistent_with_latest` is `false`, then check its consistency with `latest_sth`. If the check succeed, we set `checked_consistent_with_latest` to `true`.

If only consistency check successes but not certificate fetching, `checked_consistent_with_latest` should be kept at `false`. This way, when we retry and get a new sth, once we are done checking consistency and fetching certificates we will eventually check the sth we got last time to make sure it is also consistent. That way we make sure we check all of the sth that we received for consistency.

However, if the newly gotton sth has a `tree_size` &le; the current `latest_sth`.`tree_size`, we just add the sth to the `sth` table and do nothing, so that `latest_sth`.`tree_size` is always strictly increasing whenever we update it.

## Gossip

sth received from gossips should (after signature verification) be stored in the `sth` table with `checked_consistent_with_latest` set to `false`. Therefore, the gossiped sth will eventually be checked for consistency when we updated our tree to at least the `tree_size` of the gossiped sth. We don't check it immediately so that we can decouple the process of receiving gossip and the process of checking consistency, so that we may retry the consistency check or defer it, if the log is presenting a delayed version of itself to us.

## Invariants

For each row in `ctlogs`:

* Whenever `latest_sth` is updated from a value $s_1$ to $s_2$, the sth with id $s_1$ is consistent with $s_2$ and $s_1$.`tree_size` &le; $s_2$.`tree_size`. This is obvious.

* Let $s_1$ be any value that `latest_sth` has ever taken in the past and let $s'$ be the current `latest_sth`. The sth with id $s_1$ is consistent with that of $s'$.
  * Let $s_1, s_2, \ldots, s_n$ be a sequence of changes that the `latest_sth` value has been through from $s_1$ to $s'$, so that $s_n = s'$. Because of the first invariant, $s_k$ is consistent with $s_{k + 1}$ for any $k \in \{1, \ldots, n - 1\}$. Hence $s_1, \ldots, s_n$ is a "chain" of consistent sths. By the transitive property of "consistent", $s_1$ is consistent with $s_n$ and hence $s'$.

For all sth recorded in the `sth` table:

* `checked_consistent_with_latest` is `true` only if its `tree_size` is less than or equal to the `tree_size` of the sth pointed to by `ctlogs`.`latest_sth`, and also that we are sure it will be consistent with `latest_sth`.

	Proof:

	1. The first part &mdash; the `tree_size` of any sth with this field being `true` is &le; the `tree_size` of the `latest_sth` &mdash; is obvious, because we never check any consistency until our `latest_sth` have a large enough `tree_size`.
	2. The second part &mdash; we are confiednt that this sth is consistent with `latest_sth` &mdash; is also true. Since `checked_consistent_with_latest` for this sth is set to `true`, there must be some past value of `latest_sth` $s_1$ for which we have checked the consisteny of this sth with. Because we have proved that such a $s_1$ will be consistent with the current value of `latest_sth`, by transitivity, the sth in question is consistent with the current `latest_sth`.
