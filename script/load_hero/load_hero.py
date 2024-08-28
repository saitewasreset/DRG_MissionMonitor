import requests
import json
import sys

source_addr = "https://github.com/saitewasreset/DRG_MissionMonitor"

print("Mission Monitor: load_hero")
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

hero_list = ["DRILLER", "ENGINEER", "GUNNER", "SCOUT"]

load_hero_endpoint = "{}/load_hero".format(admin_endpoint)

print("Uploading to server...")
try:
    r = requests.post(load_hero_endpoint, json=hero_list)
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
