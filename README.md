# SALAD

The all-in-one online portal for Comet Robotics

# Getting Started

The System for Attendance, Logistics, And Dues (SALAD) manages the web app for several things at CR, including:

- Tracking members and linking their Discord and UTD accounts
- Process membership payments
- Tracking membership status per-term
- Tracking events and attendance

All of this is managed by the Comet Robotics Office of Unified Technology, Operations, and Networking (CROUTON). Please reach out to them for suggestions, concerns, or interest in joining.

# Usage

Features of this app vary heavily with the category of user:

## General Members

SALAD does not manage authentication for general members. Thus, they can only make membership payments (only valid for members who have linked their Discord and NetID) or self-check-in for events (if a QR code is provided).

## Leads

Project managers and team leads (both referred to as "leads") are responsible for tracking the attendance of all members for their respective project. They are also obligated to ensure all members have paid their dues. Therefore, SALAD has features to help them:

- Create events associated with a project
- Manually check in members (by name, NetID, or card swipe)
- Produce and display a QR code for check-in 
- View a list of all members checked in for an event associated with their respective project
- View a list of all members who have ever attended an event for their project
- Verify if the members listed above have paid dues for the relevant semester
- All features given to general members

They have permission to view view a member's name, NetID, major, paid status, and their attendance history for all events across all projects.

## Officers

All active officers have full admin permissions. Through the web app, they can:

- View all tables in the DB (members, payments, events, etc.)
- Elevate or demote members
- Define the membership fees per semester
- Officiate membership payments via cash transaction
- All features given to leaders as if they were associated with all projects

# Development

All active CR members are welcome and encouraged to maintain this project! They unfortunately cannot have access to the database, either through web or CLI, for security reasons. Instead, they must test everything locally and can only PR to main. Officers are responsible for deployment.

This codebase is written to run fully in the cloud (we've run into issues hosting it ourselves and Azure gives us a free nonprofit subscription). However, you can fully run a demo of this locally if you successfully install the build dependencies. TL;DR you only need the following:

- Rust (latest version, with cargo)
- PostgreSQL (local server)
- Solid + Typescript (with bun)

```
Insert more specific instructions here
```
