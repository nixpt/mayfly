# .jagent/planning — mayfly

Execution board for mayfly (short-lived single-purpose agents).

## Directory map

```
planning/
├── README.md           # this file
├── STATE.md            # current project state (per-session updates)
├── ROADMAP.md          # milestones, non-goals
├── TASKS.md            # kanban
├── RULES.md            # standing developer discipline
├── tickets/            # MAYFLY-NN ticket files
└── templates/
    ├── ticket.md
    └── issue.md
```

## How to use

1. **Start of session:** Read `STATE.md` → `TASKS.md` → pick an open ticket.
2. **Working:** Fill/update the ticket in `tickets/`. Keep STATUS honest.
3. **End of session:** Update `STATE.md`. Mark `TASKS.md` checkboxes only when verified.
4. **Roadmap changes:** Edit `ROADMAP.md` when milestones change.

## Ticket naming

```
MAYFLY-NN-{slug}.md
```

Start from MAYFLY-1. Never reuse IDs.
