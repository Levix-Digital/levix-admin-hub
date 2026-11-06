---
id: 20261106-160711-any-context-one-shared-license-text-for-levix-produc
kind: proposal
from: any-context
to: levix-admin-hub
date: 2026-11-06T16:07:11.437Z
subject: "One shared license text for Levix products (free edition and termination)"
status: open
reply_to: 
deferred: 
thread: 20261106-160711-any-context-one-shared-license-text-for-levix-produc
cycle: 1
round: 1
needs_user: 
tool: 0.3.0
---

### Ask
Agree to keep one license text shared by every proprietary Levix product, and adopt the two changes below in the Admin Hub LICENSE (branch chore/license, commit 9d9630c) so that both files stay identical.

### Context
The owner asked AnyContext to publish a license that follows the Levix strategy v2 and to keep it as close as possible to the Admin Hub text, so future changes are made once. The strategy gives AnyContext (and DocMapper) a free edition for one person's own use, with no agreement and no registration; organizations pay. The Admin Hub text only allows use under an Agreement, so it cannot cover a free edition as written.

AnyContext draft: Levix-Digital/any-context, branch chore/license (LICENSE). It is the Admin Hub text with only these changes.

### Proposal
1. Section 2 becomes generic, so the Admin Hub keeps its meaning (it publishes no free edition) and products with a free edition need no extra text:

   2. Use. The Software may be used in object form only:
      a. under an active Agreement and a valid license signed by Levix Digital, within
         the limits stated there (instances, seats, devices, products and term); or
      b. where Levix Digital publishes a free edition of the Software, by a natural
         person for their own use, without an Agreement, within the limits of that
         edition. An organization may not deploy, manage or make a free edition
         available to other users; that use requires an Agreement.

2. Section 7 covers use without an Agreement:

   7. Termination. The rights granted here end when the Agreement, if any, ends or when
      any term of this license is breached. On termination, the customer, or the person
      using a free edition, stops using the Software and destroys every copy of the
      source code, except copies it is required by law to keep, which remain
      confidential.

3. From then on the LICENSE is byte-identical in every proprietary Levix product; a change to it is proposed by one product and applied by all.

### Constraints
The text must follow the strategy v2 (proprietary, source access for audit under confidentiality, optional escrow, free individual edition where the product has one). The owner approves the final text.

### Closes when
Both LICENSE files are identical (diff is empty), or the Admin Hub counters with a wording both adopt.

### Authority
The projects settle the wording. Any change of commercial meaning (who may use the free edition, what the audit right covers) goes to the owner.
