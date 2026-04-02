# Friction Log

## What would you do more of if it took 10 minutes instead of 2 hours? Friction Log is a friction journal that pays out. 

Log friction moments → the app spots patterns → it produces an automation brief (a structured output). That brief is the product. It makes the value tangible and gives users a reason to keep logging.

The loop becomes:

Log friction → See pattern → Get brief → Hand off to AI agent → Reclaim time


Friction Log is a dektop application (based on Tauri v2) that complements AI Agents like Claude Cowork, OpenClaw and others, by helping you to analyse how you work and what to delegate/automate.

The application aims to help the user to identify

- Find the repetitive work
  - What do you do every week that feels like Groundhog Day?
  - What tasks do you dread because they're boring, not hard?
- Find the coordination tax
  - How much time do you spend chasing people for updates, approvals, or information?
  - What's the thing that always falls through the cracks when you're busy?
- Find the context-switching cost
  - What pulls you out of deep work most often?
- Find the bottleneck
  - What's the one thing, if it happened faster, that would unblock everything else?
  - What do people wait on you for — and what are you waiting on others for?
- Find the guilt pile
  - What's the thing you know you should be doing consistently but never get to?
  - What would you do more of if it took 10 minutes instead of 2 hours?

The "Guilt Pile" and "Coordination Tax" are excellent psychological categories that users relate to.


Every few seconds, the app asks the OS one question: "what application is currently in focus?" and stores the title, and the time.

The user can enter manually what they were doing and how. They can add notes about ideas how to solve it or which things are important for them.

Pull  →  user logs a conscious frustration
Push  →  system detects a pattern, asks one question

- Implementation Suggestion: Create a "Spotlight" style window that is transparent, centered, and has no decorations. 
  - Use app.handle().get_webview_window("main").unwrap().set_focus() when the shortcut is hit.

- The "Frustration" Input: Instead of a complex form, use a single command line with Natural Language Processing (NLP).
	- User types: "Tired of copy-pasting Jira IDs into Slack."

	- System tags: #bottleneck, #context-switch, app:Jira, app:Slack.

- The "Quick-Note" Pattern: Ensure the window disappears immediately after hitting Enter. 
  - Use an asynchronous backend task to process the note so the UI feels instantaneous.

- Tauri v2 Enhancement: Use the active-win or rev_window crate in the Rust backend to get the window title and process name.

- The "Nudge" Logic:
	- Example: If the system detects you've switched between VS Code and Stack Overflow 15 times in 10 minutes, a small toast notification appears: "High context-switching detected. Is this a research task we should document?"

- The "Wait-Time" Tracker (The Bottleneck) Idea: Add a "Waiting" toggle to your shortcut. If you hit Alt+Space and just type .w, it marks the start of a "Blocked" period. When you type .w again, it calculates the duration. This identifies the Bottleneck without you having to write a paragraph.

- Semantic Clustering of Frustrations:
  - Use the Claude Cowork/OpenClaw integration here. Periodically, the app should "digest" your logs and present a "Friction Heatmap."
    - Output: "You spent 4 hours this week on 'Coordination Tax' specifically regarding 'Project X' approvals. Here is a prompt template for an AI Agent to handle this."

- Integration with "Agentic" Output:
  - Include a "Copy as Agent Spec" button or "Send to Claude"?
  - Example:
    - "I am an engineer who spends 20% of my time on [Task X]. Here are 10 examples of how I do it. Write a script to automate the API calls between [App A] and [App B]."

- Use SQLite (via the tauri-plugin-sql) to store these events and run a cleanup job that aggregates "Focus Time" into 5-minute chunks to save space.
- In Tauri v2, ensure you handle macOS "Accessibility" permissions properly, as the OS will block you from seeing other window titles otherwise.
- Instead of a blind loop, use std::thread::sleep in a Rust Command or a dedicated background thread to ensure you aren't spiking CPU usage for an "invisible" app.

We do not capture keystrokes, screenshots, or page contents
