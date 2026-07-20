import os
import time
import subprocess
import shutil
import threading

LOG_DIR = "test_logs"
LOG_FILE = os.path.join(LOG_DIR, "latest.log")

def setup():
    if os.path.exists(LOG_DIR):
        shutil.rmtree(LOG_DIR)
    os.makedirs(LOG_DIR)
    with open(LOG_FILE, "w") as f:
        f.write("[Server thread/INFO]: Test server started\n")

def append_log(msg):
    with open(LOG_FILE, "a") as f:
        f.write(f"[Server thread/INFO]: {msg}\n")
    print(f"Appended: {msg}")

def rotate_log():
    print("Rotating log...")
    timestamp = int(time.time())
    rotated_name = os.path.join(LOG_DIR, f"{timestamp}.log")
    os.rename(LOG_FILE, rotated_name)
    with open(LOG_FILE, "w") as f:
        f.write("")
    print(f"Rotated to {rotated_name} and created new latest.log")

def run_watcher():
    env = os.environ.copy()
    env["LOG_DIRS"] = LOG_DIR
    env["CHATWORK_API_TOKEN"] = "dummy"
    env["CHATWORK_ROOM_ID"] = "dummy"
    
    # We use cargo run. We need to parse output to see if it detects things.
    # The app prints "🟢 Monitoring: ..." and "🔄 Log rotation detected..." (if it works)
    process = subprocess.Popen(
        ["cargo", "run"], 
        env=env, 
        stdout=subprocess.PIPE, 
        stderr=subprocess.PIPE,
        text=True,
        cwd="/home/hit/docker/mc_log_watcher"
    )
    return process

def main():
    setup()
    
    print("Starting watcher...")
    process = run_watcher()
    
    # Give it time to compile and start
    time.sleep(10) 
    
    print("Phase 1: Normal append")
    append_log("Player1 joined the game")
    time.sleep(2)
    
    print("Phase 2: Rotation")
    rotate_log()
    time.sleep(2)
    
    print("Phase 3: Append to new log (large write)")
    # Write event FIRST
    msg = "Player2 joined the game"
    with open(LOG_FILE, "a") as f:
        f.write(f"[Server thread/INFO]: {msg}\n")
    
    # Then write enough data to exceed previous file size
    large_padding = "X" * 2000
    with open(LOG_FILE, "a") as f:
        f.write(f"[Server thread/INFO]: Padding {large_padding}\n")
    
    print(f"Appended: {msg}")
    time.sleep(5)
    
    print("Stopping watcher...")
    process.terminate()
    try:
        stdout, stderr = process.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        stdout, stderr = process.communicate()
        
    print("--- Watcher Output ---")
    print(stdout)
    print("--- Watcher Error ---")
    print(stderr)

if __name__ == "__main__":
    main()
