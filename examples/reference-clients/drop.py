#!/usr/bin/env python3
"""
Zero-dependency reference writer to plant scents, whispers, and action vectors into .crumb.local.
Usage:
  python3 drop.py whisper "Refactoring in progress" --from CoderAgent
  python3 drop.py action modify src/lib.rs "Fix timeout" --vector resilience
"""

import argparse
import json
import os
from datetime import datetime, timezone

def load_or_create_local(directory):
    local_path = os.path.join(directory, ".crumb.local")
    if os.path.exists(local_path):
        try:
            with open(local_path, "r", encoding="utf-8") as f:
                return json.load(f), local_path
        except Exception:
            pass
    return {
        "schema_version": "1.0.0",
        "directory": os.path.relpath(directory),
        "active_scents": {},
        "locks": {},
        "whispers": [],
        "history": []
    }, local_path

def save_local(data, local_path):
    with open(local_path, "w", encoding="utf-8") as f:
        json.dump(data, f, indent=2)

def main():
    parser = argparse.ArgumentParser(description="Plant scents or whispers into .crumb.local")
    subparsers = parser.add_subparsers(dest="command")

    w_parser = subparsers.add_parser("whisper")
    w_parser.add_argument("message", help="Whisper message")
    w_parser.add_argument("--from-agent", default="AutonomousAgent", help="Agent identifier")
    w_parser.add_argument("--dir", default=".", help="Target directory")
    w_parser.add_argument("--target-file", help="Optional target file")

    a_parser = subparsers.add_parser("action")
    a_parser.add_argument("action", help="Action type (modify, create, delete, test)")
    a_parser.add_argument("target", help="Target file name")
    a_parser.add_argument("intent", help="Intent description")
    a_parser.add_argument("--vector", default="general", help="Vector description")
    a_parser.add_argument("--agent", default="AutonomousAgent", help="Agent identifier")
    a_parser.add_argument("--dir", default=".", help="Target directory")

    args = parser.parse_args()
    now_iso = datetime.now(timezone.utc).isoformat()

    if args.command == "whisper":
        data, path = load_or_create_local(args.dir)
        whisper_entry = {
            "from": args.from_agent,
            "message": args.message,
            "timestamp": now_iso
        }
        if args.target_file:
            whisper_entry["target_file"] = args.target_file
        data.setdefault("whispers", []).append(whisper_entry)
        save_local(data, path)
        print(f"Planted whisper in {path}")

    elif args.command == "action":
        data, path = load_or_create_local(args.dir)
        action_entry = {
            "agent": args.agent,
            "action": args.action,
            "target": args.target,
            "intent": args.intent,
            "vector": args.vector,
            "timestamp": now_iso
        }
        data.setdefault("history", []).append(action_entry)
        if len(data["history"]) > 20:
            data["history"] = data["history"][-20:]
        save_local(data, path)
        print(f"Recorded action vector in {path}")

if __name__ == "__main__":
    main()
