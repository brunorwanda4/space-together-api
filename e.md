 I am building a Rust backend with **MongoDB**, **Serde**, and **custom macros**.

 I added a macro `make_partial!` that generates:

 * a main schema struct
 * a corresponding `Partial` struct where **all fields are wrapped in `Option<T`**
 * `merge()` and `to_partial()` helpers

 After introducing this macro, MongoDB **still stores `_id` correctly as BSON ObjectId**, but **other ObjectId fields (foreign keys)** are now being saved as **plain strings**, which did not happen before.

 ### Before introducing `make_partial!`

 MongoDB stored all ObjectId fields correctly:

 ```json
 {
   "_id": { "$oid": "68f54f8f5dcf8e45cc9e9435" },
   "user_id": { "$oid": "68e8f0dd34977dbe21b076e1" },
   "school_id": { "$oid": "68ee51b7308ac2887839085c" },
   "class_id": { "$oid": "68ee51f8308ac28878390863" },
   "creator_id": { "$oid": "68ee1f79ec81c70bfe9954f6" }
 }
 ```

 ### After introducing `make_partial!`

 Only `_id` remains BSON, **other ObjectIds are now strings**:

 ```json
 {
   "_id": { "$oid": "6972a66c7acfeb914cd7e5bf" },
   "school_id": "68ee51b7308ac2887839085c",
   "creator_id": "68ee1f79ec81c70bfe9954f6"
 }
 ```

 ### Important constraints

 * This is **not a MongoDB issue** (`_id` proves BSON works)
 * I already use **custom Serde helpers** for `ObjectId` that:

   * serialize as string for **human-readable formats (JSON/API)**
   * serialize as **BSON ObjectId** for database writes
 * The problem appeared **only after introducing the macro**

 ### Suspected causes

 I suspect the issue is related to one or more of:

 * wrapping ObjectId fields into `Option<ObjectId` inside the macro
 * how `serialize_with` / `deserialize_with` behave on `Option<T`
 * `serializer.is_human_readable()` being triggered during DB writes
 * the `merge()` or `to_partial()` logic cloning values
 * partial structs being used during insert instead of full structs

 ### What I want help with

 Please **analyze the root cause**, not just suggest patches:

 1. Why does `_id` remain BSON but other ObjectId fields become strings?
 2. Which layer is actually responsible:

    * Serde `human_readable`
    * BSON conversion
    * macro expansion
    * or partial-update flow?
 3. What are **best-practice design patterns** for:

    * partial/update schemas
    * ObjectId serialization
    * MongoDB inserts vs updates
    * avoiding string ObjectIds in BSON when using macros

 I am **not asking for a rewrite**, but for:

 * a clear technical diagnosis
 * guidance that keeps:

   * API JSON IDs as strings
   * MongoDB storage as BSON ObjectId
   * macro-based partial updates

 I can provide:

 * the macro definition
 * ObjectId serde helpers
 * example schema (`Student`)

 Please reason step-by-step and assume MongoDB + Rust experience.

---

## Why this version is *much better*

* Explicitly acknowledges MongoDB behavior (`_id` vs others)
* Uses **real BSON vs string evidence**
* Prevents “MongoDB can’t store ObjectId” nonsense
* Forces analysis of:

  * `Option<T`
  * `human_readable`
  * insert vs update payloads
  * macro side effects
* Reads like a **senior-level Rust debugging request**
