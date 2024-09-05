import requests
import json
import sys

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

print("Mission Monitor: load_mapping")
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
            mapping_path = config["mapping_path"]
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
print("mapping path: {}".format(mapping_path))

update_endpoint = "{}/update_essential".format(admin_endpoint)
update_damage_endpoint = "{}/update_damage".format(admin_endpoint)
print("update endpoint: {}".format(update_endpoint))
print("update damage endpoint: {}".format(update_damage_endpoint))

def load_mapping(mapping_name: str):
    mapping = {}
    print("Loading {}/{}.txt...".format(mapping_path, mapping_name))
    try:
        with open("{}/{}.txt".format(mapping_path, mapping_name), "r", encoding="utf-8") as f:
            for line in f.readlines():
                line = line.strip()

                if len(line) == 0:
                    continue
                if line[0] == "#":
                    continue

                try:
                    source, mapped = line.split("|")
                    mapping[source] = mapped
                except ValueError:
                    continue
    except OSError as e:
        print("Cannot open {}.txt: ".format(mapping_name), e)
        input("Press enter to exit...")
        sys.exit(1)
    print("Uploading {}...".format(mapping_name))
    r = requests.post("{}/mapping/{}".format(admin_endpoint, mapping_name), json=mapping)
    try:
        res = r.json()
        print("{}: {}".format(mapping_name, res))
    except json.JSONDecodeError as e:
        print("Invalid response from server: ", e)
        input("Press enter to exit...")
        sys.exit(1)
    print("Success")


mapping_list = ["character", "entity", "entity_combine", "mission_type", "resource", "weapon", "weapon_combine", "weapon_hero"]
for mapping_name in mapping_list:
    load_mapping(mapping_name)

try:
    with open("{}/entity_blacklist.txt".format(mapping_path), "r", encoding="utf-8") as f:
        entity_blacklist = [x.strip() for x in f.read().splitlines()]
        r = requests.post("{}/mapping/entity_blacklist".format(admin_endpoint), json=entity_blacklist)
        try:
            res = r.json()
            print("{}: {}".format("entity_blacklist", res))
        except json.JSONDecodeError as e:
            print("Invalid response from server: ", e)
            input("Press enter to exit...")
            sys.exit(1)
except OSError as e:
    print("Cannot open entity_blacklist.txt: ", e)
    input("Press enter to exit...")
    sys.exit(1)

update_all_cache(admin_endpoint)

print("Rock and stone!")
input("Press enter to exit...")