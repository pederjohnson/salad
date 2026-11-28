# SALAD Development Roadmap

This document lists every feature of SALAD at a high level. It is meant to be referenced and edited by humans and agents alike. Each feature specifies:

- Implementation status (not started/in progress/done)
- How the user reaches this feature
	- Does it depend on the context of another feature?
- How the feature is displayed visually
	- Main content, modal, notification, etc.
- Sub-features
	- Frontend/backend (as applicable) with implementation status and, optionally, how it's implemented (endpoint, element, etc.)

Like stated in the README, remember that any feature is available per-role by a trickle-up effect (e.g. all lead features are accessible by officers), which every endpoint must uphold. Neglecting this philosophy can produce lower-privilege features that conflict with higher-privilege ones, and preventing a user from doing the functions of their subordinate is an administrative blunder.

## Signed Out

### Login Screen

- Implementation: in progress
- Reachable at `portal.cometrobotics.org` with no auth token
- Shown as modal on decorative background
- Accepts NetID as login info, sends confirmation email with 6-digit code to `<net_id>@utdallas.edu`
	- Frontend: not started
	- Backend: in progress (`POST /api/auth/code` does not send an email)
- Accepts 6-digit code and gives auth token to browser
	- Frontend: not started
	- Backend: done (as `POST /api/auth/verify`)
- Accepts any 6-digit code for debug builds
	- Backend: done (through `ACCEPT_ANY_CODE` in src/api/auth.rs)

## General Member

### Main Dashboard

- Implementation: not started
- Reachable at `portal.cometrobotics.org`
- Allows roles to have "dashboard features" for quick insights, pressing actions, etc.

### User Menu

- Implementation: in progress
- Reachable by clicking right-most button of nav menu
- Shown as drop-down menu of button
- Greets user by name
	- Frontend: not started
	- Backend: done (as `GET /api/auth/me`)
- User role menu item
	- Frontend: not started
	- Backend: done (as `GET /api/auth/me`)
- "Log Out" menu item
	- Frontend: not started
	- Backend: done (as `POST /api/auth/logout`)

## Officer

### Full Members List

- Implementation: in progress
- Reachable by clicking "Members" in nav bar
- Shown as main content in table form
	- Columns: NetID, name (first + last), paid dues for current semester, role
	- As applicable, columns can be sorted or filtered by clicking on the header row
	- Above the header row is a "New" button, a text search box, and buttons for filtering/sorting when not related to a column
	- Filtering/sorting are done client-side whenever possible
- Displays all members listed in our database
	- Frontend: not started
	- Backend: done (as `GET /api/members`)
- Sort by NetID
	- Frontend: not started
- Sort by Name (first or last)
	- Frontend: not started
- Filter by paid status
	- Frontend: not started
- Filter by whether they've been active in any events this semester
	- Frontend: not started
	- Backend: not started
- Fuzzy search NetID and name
	- Frontend: not started
- "New" button launches a new member modal
	- Frontend: not started
	- Backend: not started
	- TO DO: "new member modal" feature not specified!

### Member Modal

- Implementation: in progress
- Reachable by clicking on a row in a members list table
- Shown as modal above members table
- Displays:
	- Every field of the member (as stored in database)
	- Their role (and project if lead)
	- Past terms paid for
	- A summary of past payments they have made (currently just term payments)
	- Frontend: not started
	- Backend: done (as `GET /api/members/{net_id}`)
- Can perform edits of NetID, name, major, and Comet Card ID
	- Frontend: not started
	- Backend: done (as `PATCH /api/members/{net_id}`)

### Staff & Projects Panel

- Implementation: not started
- Reachable by clicking "Staff & Projects" in nav bar
- TO DO: describe this more, later
- Changing role and submitting adds/removes/updates the row in the database.
	- Frontend: not started
	- Backend: not started (expect `POST /api/staff/{net_id}`)

### Terms Table

- Implementation: not started
- Reachable by clicking "Terms" in nav bar
- Shown as main content in table form
- Columns: Semester, Effective, Normal Fee, Due, Late Fee. Fees are converted from integer cents to decimal USD.
	- Frontend: not started
	- Backend: not started (expect `GET /api/terms`)
- Extra column "Payments" that displays how many payments happened in that term's payment window
	- Frontend: not started
	- Backend: not started (expect `GET /api/terms`)
- Last row has a "New" button:
	- Clicking adds a blank row
	- Becomes "Confirm" after adding blank row or when there are any pending edits
	- Clicking either creates the new term or saves the modified term
	- Successful or cancelled edit reverts button back to "New"
	- Frontend: not started
- Rows can be edited by clicking on a cell:
	- Text input for semester name
	- Date inputs for dates
	- Number input that is edited in cents but displayed in dollars when deselected (e.g. $15.00 → 1500 → 2500 → $25.00)
	- Only one row may be edited at a time. This includes a row created by clicking "New". Attempted edits to other rows notify this to the user.
	- "Cancel" button takes place of cell under "Payments"
	- Works without backend
	- Frontend: not started
- The edited row communicates all constraints in the SQL table for terms. Happens after pressing "Confirm" (client-side, interrupts request) and after receiving an error message (just to be safe).
	- Frontend: not started
	- Backend: not started (requires adding to `constraint_message` in src/api/error.rs)
