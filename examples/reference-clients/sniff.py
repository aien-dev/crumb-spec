#!/usr/bin/env python3
"""
Zero-dependency reference reader for the Crumb Protocol (RFC-0001).
Usage: python3 sniff.py [path/to/directory]
"""

import json
import os
import sys

def sniff_directory(path="."):
    target_dir = os.path.abspath(path)
    if not os.path.isdir(target_dir):
        target_dir = os.path.dirname(target_dir)

    crumb_path = os.path.join(target_dir, ".crumb")
    crumb_local_path = os.path.join(target_dir, ".crumb.local")

    print(f"--- Crumb Topography for: {target_dir} ---")

    if os.path.exists(crumb_path):
        try:
            with open(crumb_path, "r", encoding="utf-8") as f:
                crumb = json.load(f)
            print(f"Module: {crumb.get("name", "unnamed")} (Layer: {crumb.get("layer", "n/a")})")
            print(f"Purpose: {crumb.get("purpose", "n/a")}")
            
            above = crumb.get("above")
            if above:
                if isinstance(above, dict):
                    print(f"Above: {above.get("name")} -> {above.get("path")}")
                else:
                    print(f"Above: {above}")

            invariants = crumb.get("invariants", [])
            if invariants:
                print("Invariants:")
                for inv in invariants:
                    print(f"  * {inv}")
        except Exception as e:
            print(f"Error reading .crumb: {e}")
    else:
        print("No .crumb found in this directory.")

    if os.path.exists(crumb_local_path):
        try:
            with open(crumb_local_path, "r", encoding="utf-8") as f:
                local = json.load(f)
            
            scents = local.get("active_scents", {})
            if scents:
                print("\nActive Agent Scents:")
                for agent, s in scents.items():
                    print(f"  [{agent}] Focus: {s.get("focus")} (Updated: {s.get("updated_at")})")

            locks = local.get("locks", {})
            if locks:
                print("\nActive Resource Locks:")
                for res, lock in locks.items():
                    print(f"  [LOCK] {res} held by {lock.get("holder")} (Intent: {lock.get("intent")})")

            whispers = local.get("whispers", [])
            if whispers:
                print("\nRecent Whispers:")
                for w in whispers[-5:]:
                    print(f"  From {w.get("from")}: \"{w.get("message")}\" ({w.get("timestamp")})")

            history = local.get("history", [])
            if history:
                print("\nRecent History Vectors:")
                for h in history[-3:]:
                    print(f"  {h.get("timestamp")} | {h.get("agent")} {h.get("action")} {h.get("target")} - {h.get("intent")}")
        except Exception as e:
            print(f"Error reading .crumb.local: {e}")
    else:
        print("\nNo .crumb.local detected (no active agent scents).")

if __name__ == "__main__":
    target = sys.argv[1] if len(sys.argv) > 1 else "."
    sniff_directory(target)
