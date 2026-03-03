# Manual test: permission-based live updates

To verify that live updates work when one user mutates and another views:

1. Start the backend and frontend (with `VITE_BACKEND_URL` set in dev so the frontend proxies `/api` and `/ws` to the backend).
2. Log in as **User A** (e.g. Alice) in one browser (or tab).
3. Log in as **User B** (e.g. Bob) in another browser (or incognito).
4. Ensure both have access to the same org (e.g. switch to the same organization if the app has org switching).
5. **Bob** opens a page that lists data (e.g. Users, Roles, or Tasks).
6. **Alice** creates or updates a record (e.g. add a user, edit a role).
7. **Bob**’s page should update automatically (list refetches or task list updates) without refresh, as long as Bob has permission to view that resource and the server has subscribed his WebSocket to the corresponding channel (see docs/021).

If the WebSocket is connected (e.g. “Live” chip visible), updates should appear at least once. Reconnection after network drop is handled by the client (single WS re-opens when session is still valid).
