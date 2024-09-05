import requests
import csv
import sys
import json

def print_endpoint(endpoint: str, name: str):
    print("update {} endpoint: {}".format(name, endpoint))

def update_cache(endpoint: str, name: str):
    print("Updating {} cache...".format(name))
    r = requests.get(endpoint)
    try:
        res = r.json()
        if res["code"] != 200:
            print("Server returned an error:  ", res)
            input("Press enter to exit...")
            sys.exit(1)
        else:
            print("Success! time: {}ms".format(res["data"]["time_ms"]))
    except json.JSONDecodeError:
        print("Invalid response from server: ", r.text)
        input("Press enter to exit...")
        sys.exit(1)

def update_all_cache(admin_endpoint: str):
    update_mission_kpi_endpoint = "{}/update_mission_kpi".format(admin_endpoint)
    update_endpoint = "{}/update_essential".format(admin_endpoint)
    update_damage_endpoint = "{}/update_damage".format(admin_endpoint)
    update_general_endpoint = "{}/update_general".format(admin_endpoint)

    update_list = [(update_mission_kpi_endpoint, "mission kpi"), (update_endpoint, "essential"), (update_damage_endpoint, "damage"), (update_general_endpoint, "general")]

    for endpoint, name in update_list:
        print_endpoint(endpoint, name)
    for endpoint, name in update_list:
        update_cache(endpoint, name)

source_addr = "https://github.com/saitewasreset/DRG_MissionMonitor"

print("Mission Monitor: load_kpi")
print("made by saitewasreset with love")
print("Source: {}".format(source_addr))

print()
print("Afraid of the dark? No need, you got me!")
print()

print("Loading config.json...")

try:
    with open("../config.json", "r", encoding="utf-8") as f:
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

character_id_to_game_id = ["NONE", "DRILLER", "ENGINEER", "GUNNER", "SCOUT"]

result = {
    "version": "0.2.2",
    "priorityTable": {},
    "character": {}
}

try:
    with open("kpi_data.txt", "r", encoding="utf-8") as f:
        for line in [x.strip() for x in f.readlines()]:
            if len(line) == 0:
                continue
            if line[0] == "#":
                continue

            line_data = line.split(" ")

            character_game_id = character_id_to_game_id[int(line_data[0])]
            character_subtype_id = int(line_data[1])
            character_subtype_name = line_data[2]
            character_weight_list = [float(x) for x in line_data[3:]]

            result["character"].setdefault(character_game_id, {})[character_subtype_id] = {
                "subtypeName": character_subtype_name,
                "weightList": character_weight_list,
                "priorityTable": {
                    "default": 1.0
                }
            }
except Exception as e:
    print("Cannot parsing kpi_data.txt: ", e)
    input("Press enter to exit...")
    sys.exit(1)

try:
    with open("entity_list_combined.csv", "r", encoding="utf-8") as f:
        reader = csv.reader(f)
        reader_iter = reader.__iter__()
        # skip table header
        reader_iter.__next__()

        result["priorityTable"]["default"] = 0.0

        for row in reader_iter:
            entity_game_id = row[0]
            priority_weight = float(row[2])
            character_weight_list = [float(x) for x in row[3:]]

            if float(priority_weight) != 0.0:
                result["priorityTable"][entity_game_id] = float(priority_weight)

            i_map = [("DRILLER", 1), ("GUNNER", 1), ("ENGINEER", 1), ("SCOUT", 1), ("SCOUT", 2)]
            for i, character_weight in enumerate(character_weight_list):
                if character_weight != 1.0:
                    result["character"][i_map[i][0]][i_map[i][1]]["priorityTable"][entity_game_id] = character_weight
except Exception as e:
    print("Cannot parsing entity_list_combined.csv: ", e)
    input("Press enter to exit...")
    sys.exit(1)

kpi_endpoint = "{}/kpi".format(admin_endpoint)
update_endpoint = "{}/update_essential".format(admin_endpoint)

print("kpi endpoint: {}".format(kpi_endpoint))
print("update endpoint: {}".format(update_endpoint))

print("Uploading kpi...")
try:
    r = requests.post(kpi_endpoint, json=result)
except requests.exceptions.RequestException as e:
    print("Request failed: ", e)
    input("Press enter to exit...")
    sys.exit(1)

try:
    res = r.json()
except json.JSONDecodeError as e:
    print("Invalid response from server: ", e)
    input("Press enter to exit...")
    sys.exit(1)

if res["code"] == 200:
    print("Success!")
    print("Updating cache...")
    r = requests.get(update_endpoint)
    try:
        res = r.json()
        if res["code"] != 200:
            print("Server returned an error:  ", res)
            input("Press enter to exit...")
            sys.exit(1)
            
    except json.JSONDecodeError as e:
        print("Invalid response from server: ", e)
        input("Press enter to exit...")
        sys.exit(1)
else:
    print("Server returned an error:  ", res)
    input("Press enter to exit...")
    sys.exit(1)

update_all_cache(admin_endpoint)

print("Success")
print("Rock and stone!")
input("Press enter to exit...")