use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogMissionInfo {
    pub begin_timestamp: i64,
    pub mission_time: i16,
    pub mission_type_id: String,
    pub hazard_id: i16,
    pub result: i16,
    pub reward_credit: f64,
    pub total_supply_count: i16,
}
// {player}|{hero}|{PlayerRank}|{CharacterRank}|{CharacterPromotionTimes}|{join}|{left}|{present}|{Kills}|{Revived}|{Deaths}|{GoldMined}|{MineralsMined}|{XPGained}|{Escaped}|{PresentAtEnd}
#[derive(Debug, Serialize, Deserialize)]
pub struct LogPlayerInfo {
    pub player_name: String,
    pub character: String,
    pub player_rank: i16,
    pub character_rank: i16,
    pub character_promotion: i16,
    pub join_mission_time: i16,
    pub left_mission_time: i16,
    pub total_present_time: i16,
    pub kill_num: i16,
    pub revive_num: i16,
    pub death_num: i16,
    pub gold_mined: f64,
    pub minerals_mined: f64,
    pub player_escaped: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogDamageInfo {
    pub mission_time: i16,
    pub damage: f64,
    pub taker: String,
    pub causer: String,
    pub weapon: String,
    // 0 -> unknown, 1 -> player, 2 -> enemy
    pub causer_type: i16,
    // 0 -> unknown, 1 -> player, 2 -> enemy
    pub taker_type: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogKillInfo {
    pub mission_time: i16,
    pub player_name: String,
    pub killed_entity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogResourceInfo {
    pub mission_time: i16,
    pub player_name: String,
    pub resource: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogSupplyInfo {
    pub mission_time: i16,
    pub player_name: String,
    pub ammo: f64,
    pub health: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogContent {
    pub mission_info: LogMissionInfo,
    pub player_info: Vec<LogPlayerInfo>,
    pub damage_info: Vec<LogDamageInfo>,
    pub kill_info: Vec<LogKillInfo>,
    pub resource_info: Vec<LogResourceInfo>,
    pub supply_info: Vec<LogSupplyInfo>,
}

impl TryFrom<&str> for LogMissionInfo {
    type Error = String;

    // NOTE: Work around for BUG in Mission Monitor MOD:
    // 在MOD中，对于深潜，每阶段结束时产生的日志中，任务时间将为已完成阶段之和，需要手动根据本局玩家的加入时间进行修正。

    /// NOTE: `value` must be full file cotent of log, not mission info part
    /// (workaround for a BUG in Mission Monitor MOD).
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let file_part_list = value.split("______").collect::<Vec<&str>>();

        let mission_info_part = *file_part_list
            .get(0)
            .ok_or(format!("missing mission info part in log: {}", value))?;

        let mission_info_split_row = mission_info_part.trim().split('|').collect::<Vec<&str>>();

        let player_info_part = *file_part_list
            .get(1)
            .ok_or(format!("missing player info part in log: {}", value))?;

        let mut min_player_join_time = i16::MAX;
        let mut player_escaped_count = 0;
        for player_info_line in player_info_part.trim().split('\n') {
            let player_info_line = player_info_line.trim();

            let player_info_split_row = player_info_line.split('|').collect::<Vec<&str>>();

            let player_info_split_row = fix_player_info_split_row(player_info_split_row)?;

            let player_join_time = player_info_split_row[5]
                .split(',')
                .collect::<Vec<&str>>()
                .join("")
                .parse::<i16>()
                .map_err(|e| format!("cannot parse player join time: {}", e))?;

            // We save player escaped count as workaround for a bug in Mission Monitor Mod.
            // See notes at calcuation of mission result below for detail.
            let player_escaped = player_info_split_row[14] == "1";
            if player_escaped {
                player_escaped_count += 1;
            }

            if player_join_time < min_player_join_time {
                min_player_join_time = player_join_time;
            }
        }

        if mission_info_split_row.len() != 7 {
            return Err(format!("element count of row is not 7: {}", value));
        }

        let begin_timestamp = mission_info_split_row[0]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i64>()
            .map_err(|e| format!("cannot parse begin timestamp: {}", e))?;

        let mission_time = mission_info_split_row[1]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse mission time: {}", e))?
            - min_player_join_time;

        let mission_type_id = remove_appendix(mission_info_split_row[2]);

        let hazard_bonus = mission_info_split_row[3]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse hazard bonus: {}", e))?;

        let mission_aborted = mission_info_split_row[4] == "2";

        // Due to a bug in game API, we cannot check whether mission is completed or failed directly
        // Instead, we check if there exists sucessfully escaped player

        let result: i16 = match mission_aborted {
            true => 2,
            false => match player_escaped_count {
                0 => 1,
                _ => 0,
            },
        };

        let reward_credit = mission_info_split_row[5]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse reward credit: {}", e))?;

        let total_supply_count = mission_info_split_row[6]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse supply count: {}", e))?;
        Ok(LogMissionInfo {
            begin_timestamp,
            mission_time,
            mission_type_id: mission_type_id.into(),
            hazard_id: get_hazard_id(hazard_bonus),
            result,
            reward_credit,
            total_supply_count,
        })
    }
}

fn get_hazard_id(hazard_bonus: f64) -> i16 {
    const EPISILON: f64 = 1e-3;
    if (hazard_bonus - 0.25).abs() < EPISILON {
        return 1;
    } else if (hazard_bonus - 0.5).abs() < EPISILON {
        return 2;
    } else if (hazard_bonus - 0.75).abs() < EPISILON {
        return 3;
    } else if (hazard_bonus - 1.0).abs() < EPISILON {
        return 4;
    } else if (hazard_bonus - 1.33).abs() < EPISILON {
        return 5;
    } else {
        return 6;
    }
}

impl TryFrom<&str> for LogPlayerInfo {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let player_info_split_row = value.split('|').collect::<Vec<&str>>();

        let player_info_split_row = fix_player_info_split_row(player_info_split_row)?;

        // {player}|{hero}|{PlayerRank}|{CharacterRank}|{CharacterPromotionTimes}|{join}|{left}|{present}|{Kills}|{Revived}|{Deaths}|{GoldMined}|{MineralsMined}|{XPGained}|{Escaped}|{PresentAtEnd}

        let player_name = String::from(player_info_split_row[0]);
        let character = String::from(player_info_split_row[1]);
        let player_rank = player_info_split_row[2]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse player rank: {}", e))?;
        let character_rank = player_info_split_row[3]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse character rank: {}", e))?;
        let character_promotion = player_info_split_row[4]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse character promotion: {}", e))?;
        let join_mission_time = player_info_split_row[5]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse join mission time: {}", e))?;
        let left_mission_time = player_info_split_row[6]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse left mission time: {}", e))?;

        let total_present_time = player_info_split_row[7]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse total present time: {}", e))?;

        let kill_num = player_info_split_row[8]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse kill num: {}", e))?;

        let revive_num = player_info_split_row[9]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse revive num: {}", e))?;

        let death_num = player_info_split_row[10]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse death num: {}", e))?;

        let gold_mined = player_info_split_row[11]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse gold mined: {}", e))?;

        let minerals_mined = player_info_split_row[12]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse minerals mined: {}", e))?;

        let player_escaped = player_info_split_row[14] == "1";

        Ok(LogPlayerInfo {
            player_name,
            character,
            player_rank,
            character_rank,
            character_promotion,
            join_mission_time,
            left_mission_time,
            total_present_time,
            kill_num,
            revive_num,
            death_num,
            gold_mined,
            minerals_mined,
            player_escaped,
        })
    }
}

/// Workaround for API change in Mission Monitor Mod：
/// 在某些版本的Mission Monitor Mod中，玩家信息不含{join}、{left}数据，此时我们默认join = 0
/// returned player_info_split_row always has 16 elements
fn fix_player_info_split_row(mut split_row: Vec<&str>) -> Result<Vec<&str>, String> {
    // Len = 14 {player}|{hero}|{PlayerRank}|{CharacterRank}|{CharacterPromotionTimes}|{present}|{Kills}|{Revived}|{Deaths}|{GoldMined}|{MineralsMined}|{XPGained}|{Escaped}|{PresentAtEnd}
    // (returned)
    // Len = 16 {player}|{hero}|{PlayerRank}|{CharacterRank}|{CharacterPromotionTimes}|{join}|{left}|{present}|{Kills}|{Revived}|{Deaths}|{GoldMined}|{MineralsMined}|{XPGained}|{Escaped}|{PresentAtEnd}

    if split_row.len() == 16 {
        Ok(split_row)
    } else if split_row.len() == 14 {
        split_row.insert(5, "0");
        split_row.insert(5, "0");
        Ok(split_row)
    } else {
        Err(format!(
            "invalid player info element of length {}: {:?}",
            split_row.len(),
            split_row
        ))
    }
}

impl TryFrom<&str> for LogDamageInfo {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let damage_split_row = value.trim().split('|').collect::<Vec<&str>>();

        if damage_split_row.len() != 9 {
            return Err(format!("element count of row is not 9: {}", value));
        }

        let mission_time = damage_split_row[0]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse mission time: {}", e))?;

        // Workaround for a bug in Mission Monitor Mod:
        // Sometimes there exists split character ',' in damage string
        let damage = damage_split_row[1]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse damage: {}", e))?;

        let is_causer_player = damage_split_row[5] == "1";
        let is_taker_player = damage_split_row[6] == "1";
        let mut is_causer_enemy = damage_split_row[7] == "1";
        let mut is_taker_enemy = damage_split_row[8] == "1";

        let record_damage_taker = damage_split_row[2];
        let record_damage_causer = damage_split_row[3];

        let mut damage_taker = String::from(record_damage_taker);
        let mut damage_causer = String::from(record_damage_causer);

        // Note: a bug in Mission Monitor Mod makes `is_*_enemy` unreliable
        // 所以，通过record_damage_*的形式判断是否为enemy：`ENE_(.+)_C` -> is_*_enemy = true

        // transform taker/causer name:
        // 1. ENE_(.+)_C -> ED_{}
        // 2. (.+)_C -> {}_C
        // Note that when only transform name for !player

        if !is_taker_player {
            let (taker_enemy, taker) = transform_record_entity_name(record_damage_taker);

            is_taker_enemy = taker_enemy;
            damage_taker = taker;
        }

        if !is_causer_player {
            let (causer_enemy, causer) = transform_record_entity_name(record_damage_causer);

            is_causer_enemy = causer_enemy;
            damage_causer = causer;
        }

        let mut causer_type: i16 = 0;
        let mut taker_type: i16 = 0;

        if is_causer_player {
            causer_type = 1;
        }

        if is_causer_enemy {
            causer_type = 2;
        }

        if is_taker_player {
            taker_type = 1;
        }

        if is_taker_enemy {
            taker_type = 2;
        }

        let mut record_weapon = damage_split_row[4];

        // WPN_Pickaxe_* (e.g. WPN_Pickaxe_Scout) -> WPN_Pickaxe_C
        if record_weapon.len() >= 12 && &record_weapon[..11] == "WPN_Pickaxe" {
            record_weapon = "WPN_Pickaxe_C";
        }

        // Note: typo "Unkown" in Mission Monitor Mod
        if record_weapon == "Unkown" || record_weapon == "" || record_weapon == record_damage_causer
        {
            record_weapon = "Unknown";
        }

        let weapon = String::from(remove_appendix(record_weapon));

        Ok(LogDamageInfo {
            mission_time,
            damage,
            taker: damage_taker,
            causer: damage_causer,
            weapon,
            causer_type,
            taker_type,
        })
    }
}

impl LogDamageInfo {
    pub fn combine_eq(&self, other: &LogDamageInfo) -> bool {
        return (self.causer_type == other.causer_type)
            && (self.taker_type == other.taker_type)
            && (self.causer == other.causer)
            && (self.taker == other.taker)
            && (self.weapon == other.weapon)
            && ((self.mission_time - other.mission_time).abs() < 5);
    }
}

fn remove_appendix(source: &str) -> &str {
    if source.len() > 3 && &source[source.len() - 2..] == "_C" {
        return &source[..source.len() - 2];
    } else {
        return &source;
    }
}

/// source -> (is_record_enemy, transformed_name)
fn transform_record_entity_name(source: &str) -> (bool, String) {
    // ENE_(.+)_C
    if source.len() > 6 && &source[..4] == "ENE_" {
        return (true, format!("ED_{}", &source[4..source.len() - 2]));
    } else {
        return (false, String::from(remove_appendix(source)));
    }
}

impl TryFrom<&str> for LogKillInfo {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let kill_split_row = value.trim().split('|').collect::<Vec<&str>>();

        if kill_split_row.len() != 3 {
            return Err(format!("element count of row is not 3: {}", value));
        }

        let mission_time = kill_split_row[0]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse mission time: {}", e))?;

        let player_name = String::from(kill_split_row[1]);
        let (_, killed_entity) = transform_record_entity_name(kill_split_row[2]);

        Ok(LogKillInfo {
            mission_time,
            player_name,
            killed_entity,
        })
    }
}

impl TryFrom<&str> for LogResourceInfo {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let resource_info_row = value.split('|').collect::<Vec<&str>>();

        if resource_info_row.len() != 4 {
            return Err(format!("element count of row is not 4: {}", value));
        }

        let mission_time = resource_info_row[0]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse mission time: {}", e))?;

        let player_name = String::from(resource_info_row[1]);
        let resource = String::from(resource_info_row[2]);
        let amount = resource_info_row[3]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse resource amount: {}", e))?;

        Ok(LogResourceInfo {
            mission_time,
            player_name,
            resource,
            amount,
        })
    }
}

impl TryFrom<&str> for LogSupplyInfo {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let resource_info_row = value.trim().split('|').collect::<Vec<&str>>();

        if resource_info_row.len() != 4 {
            return Err(format!("element count of row is not 4: {}", value));
        }

        let mission_time = resource_info_row[0]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<i16>()
            .map_err(|e| format!("cannot parse mission time: {}", e))?;

        let player_name = String::from(resource_info_row[1]);

        let ammo = resource_info_row[2]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse ammo percent: {}", e))?;

        let health = resource_info_row[3]
            .split(',')
            .collect::<Vec<&str>>()
            .join("")
            .parse::<f64>()
            .map_err(|e| format!("cannot parse health percent: {}", e))?;

        Ok(LogSupplyInfo {
            mission_time,
            player_name,
            ammo,
            health,
        })
    }
}
