import sys
import requests
import os
import re
import json

source_addr = "https://github.com/saitewasreset/DRG_MissionMonitor"

print("Mission Monitor: load_mission")
print("made by saitewasreset with love")
print("Source: {}".format(source_addr))

print()
print("Afraid of the dark? No need, you got me!")
print()

print("Loading config.json...")

try:
    with open("config.json", "r", encoding="utf-8") as f:
        try:
            config = json.load(f)
            admin_endpoint = config["admin_endpoint"]
            log_path = config["log_path"]
        except KeyError as e:
            print("Invalid config.json: cannot get required key: ", e)
            input("Press enter to exit...")
            sys.exit(1)
        except json.JSONDecodeError as e:
            print("Invalid config.json: ", e)
            input("Press enter to exit...")
            sys.exit(1)
except OSError as e:
    print("Cannot open config.json: ", e)
    input("Press enter to exit...")
    sys.exit(1)

already_uploaded_endpoint = "{}/mission_list".format(admin_endpoint)
upload_endpoint = "{}/load_mission".format(admin_endpoint)
update_endpoint = "{}/update_essential".format(admin_endpoint)
update_damage_endpoint = "{}/update_damage".format(admin_endpoint)

print("log path: {}".format(log_path))
print("already uploaded endpoint: {}".format(already_uploaded_endpoint))
print("upload endpoint: {}".format(upload_endpoint))
print("update endpoint: {}".format(update_endpoint))
print("update damage endpoint: {}".format(update_damage_endpoint))

to_load_list = []

print("Fetching already uploaded data...")

r = requests.get(already_uploaded_endpoint)
already_uploaded = r.json()["data"]

print("Reading log files...")

for filename in os.listdir(log_path):
    matched = re.match(".+_([0-9]+).txt", filename)
    if matched:
        timestamp_str = matched.group(1)
        if int(timestamp_str) not in already_uploaded:
            file_path = "{}/{}".format(log_path, filename)
            to_load_list.append((int(timestamp_str), file_path))

print("To load mission count: {}".format(len(to_load_list)))

upload_data = []

for timestamp, file_path in sorted(to_load_list, key=lambda x: x[0]):
    try:
        with open(file_path, "r", encoding="utf-16-le") as f:
            timestamp_str = re.search(r"MissionMonitor_([0-9].+)\.txt", file_path).group(1)
            # skip beginning \ufeff
            log = f.read()
            if log[0] == "\ufeff":
                log = log[1:]
            upload_data.append((timestamp_str, log))
    except UnicodeError:
        print("Cannot decode using utf-16-le: {}".format(file_path))
        print("Trying utf-8...")
        with open(file_path, "r", encoding="utf-8") as f:
            timestamp_str = re.search(r"MissionMonitor_([0-9].+)\.txt", file_path).group(1)
            # skip beginning \ufeff
            log = f.read()
            if log[0] == "\ufeff":
                log = log[1:]
            upload_data.append((timestamp_str, log))

print("Uploading...")
r = requests.post(upload_endpoint, json=upload_data)
try:
    res = r.json()
    if res["code"] != 200:
        print("Server returned an error:  ", res)
        input("Press enter to exit...")
        sys.exit(1)
    else:
        print("Success!")
except json.JSONDecodeError:
    print("Invalid response from server: ", r.text)
    input("Press enter to exit...")
    sys.exit(1)

print("Updating essential cache...")
r = requests.get(update_endpoint)
try:
    res = r.json()
    if res["code"] != 200:
        print("Server returned an error:  ", res)
        input("Press enter to exit...")
        sys.exit(1)
    else:
        print("Success!")
except json.JSONDecodeError:
    print("Invalid response from server: ", r.text)
    input("Press enter to exit...")
    sys.exit(1)

print("Updating damage cache...")
r = requests.get(update_damage_endpoint)
try:
    res = r.json()
    if res["code"] != 200:
        print("Server returned an error:  ", res)
        input("Press enter to exit...")
        sys.exit(1)
    else:
        print("Success!")
except json.JSONDecodeError:
    print("Invalid response from server: ", r.text)
    input("Press enter to exit...")
    sys.exit(1)

print("Rock and stone!")
input("Press enter to exit...")