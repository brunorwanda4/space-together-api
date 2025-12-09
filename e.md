
I need you to generate the complete logic that assigns weekly periods to each subject and maps them into a class timetable. Use the following detailed rules and requirements:

---

## 1. Subject Load → Weekly Period Allocation

* **High-load subjects (Core)**

  * Annual hours: **100–120 hrs**
  * Credits: **≥ 60**
  * Weekly periods: **8**

* **Medium-load subjects**

  * Annual hours: **70–99 hrs**
  * Credits: **40–59**
  * Weekly periods: **6**

* **Low-load subjects**

  * Annual hours: **40–69 hrs**
  * Credits: **20–39**
  * Weekly periods: **4**

* **Very low-load subjects**

  * Annual hours: **0–39 hrs**
  * Credits: **0–19**
  * Weekly periods: **2**

* **Practical-heavy TVET subjects**

  * If practical_hours ≥ 60% of annual_hours → **add +1 period**

* **National exam subjects**

  * If is_exam_subject = true → **add +1 period during exam terms**

Output a final `weekly_periods` count for each subject.

---

## 2. Period Placement Rules

After calculating weekly periods for each subject, place them into the timetable following these constraints:

### A. Class Availability

Each class has defined school operating times per day:

* `start_time`
* `end_time`
* Breaks and lunch blocks:

  * `morning_break`
  * `lunch_break`
  * `afternoon_break` (optional, not all classes use this)

These non-study blocks **cannot contain any subject periods**.

### B. Forbidden Times

Some classes or schools have time restrictions:

* Subjects cannot be placed during forbidden hours (e.g., **no afternoon study**, **no Friday afternoon**, **no late classes**).
* The system must skip these automatically.

### C. Free Periods

If a class does not have enough subjects to fill the day:

* Unallocated times must become **"free periods"**.
* Free periods must be marked as `"free_time"` with a valid start and duration.

### D. No Overlapping

Changing one period time must automatically shift non-fixed periods without breaking:

* Breaks
* Lunch
* Forbidden times
* Existing periods

### E. Teacher Constraints

A teacher may teach multiple classes. The logic must:

* Prevent a teacher from being assigned **two periods at the same time**.
* Skip or move the period to the next available slot.

---

## 3. Weekly Generation Logic

For each weekday:

1. Determine available time slots (subtract breaks + forbidden windows).
2. Insert subjects based on required `weekly_periods`, spreading them across the week evenly.
3. If a day has no available space → skip to next day.
4. Mark unused slots as `"free_time"`.

---

## 4. Expected Output

Generate a clean, structured logic description or code that does the following:

* Input:

  * List of subjects with annual hours, credits, practical %, exam flag
  * School time configuration (start, end, breaks)
  * Forbidden times
  * Teachers assigned to subjects

* Output:

  * For each subject: computed `weekly_periods`
  * For each class: a `weekly_schedule` containing:

    * day
    * periods
    * subject/break/lunch/free_time blocks
    * valid start times and durations
    * no overlaps
    * no teacher conflicts
    * auto-generated free periods

Make the solution extremely robust.
