import requests
import json
import sys

source_addr = "https://github.com/saitewasreset/DRG_MissionMonitor"

print("Mission Monitor: load_friends")
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

print("admin endpoint: {}".format(admin_endpoint))

friends_list = []

try:
    with open("friends.txt", "r", encoding="utf-8") as f:
        for line in f.readlines():
            player_name = line.strip()
            friends_list.append(player_name)
except OSError as e:
    print("Cannot reading friends.txt: ", e)
    input("Press enter to exit...")
    sys.exit(1)

load_friends_endpoint = "{}/load_friends".format(admin_endpoint)

print("Uploading to server...")
try:
    r = requests.post(load_friends_endpoint, json=friends_list)
except requests.exceptions.RequestException as e:
    print("Request failed: ", e)
    input("Press enter to exit...")
    sys.exit(1)

try:
    res = r.json()
    if res["code"] == 200:
        print("Success!")
        print("Rock and stone!")
    else:
        print("Upload failed: ", res["message"])
except json.JSONDecodeError as e:
    print("Invalid response: ", e)
    input("Press enter to exit...")
    sys.exit(1)
