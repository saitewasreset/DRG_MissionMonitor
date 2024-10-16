# API v0.12.0

## 相比 v0.8.0 变更：

### 概览（`./api/general`）

- “之前”与“最近”的分界线调整为所有有效局数 \* 0.2；
- `./player`新增历史数据；
- `./character_info`角色选择次数属性名变为`characterChoiceCount`；
- `./character`的`CharacterGeneralInfo`中按角色数据属性名变更为`characterData`；
- `./character`的`CharacterGeneralData`中总计玩家指数属性名变更为`playerIndex`。

### 伤害（`./api/damage`）

- `./`新增历史数据；
- `./character`的`CharacterDamageInfo`中总计玩家指数属性名变更为`playerIndex`。

### 任务（`./api/mission`）

- `./<int:mission_id>/general`的`MissionGeneralPlayerInfo`中角色 ID 属性名变更为`characterGameId`；
- `./<int:mission_id>/weapon`的`WeaponDamageInfo`中角色 ID 属性名变更为`characterGameId`；
- `./<int:mission_id>/kpi`的`MissionKPIInfo`：1、移除了`subtypeId`、`subtypeName`，与`heroGameId`合并为`kpi_character_type`；2、资源采集量将加权，故更名为`weightedResource`；3、原始 KPI 更名为`missionKPI`；4、大幅修改了`KPIComponent`的结构。

### KPI（`./api/kpi`）

- `./`：移除了角色信息；
- 移除了角色修正因子`./raw_data_by_promotion`（详见 KPI v0.3.0）；
- `./gamma`：1、`GammaInnerInfo`的 key 改为 kpiCharacterType；2、总计玩家指数属性名变更为`playerIndex`；3、`value`属性的含义从总值变为平均值；

- `./weight_table`：`WeightTableData`中角色 ID 变为 kpiCharacterType 的表述方法；

- `./player_kpi`：1、总计玩家指数属性名变更为`playerIndex`；2、玩家总体 KPI 属性名变更为`playerKPI`；3、大幅修改了`PlayerCharacterKPIInfo`的结构；4、`MissionKPIInfo`中原始 KPI 更名为`missionKPI`；5、`MissionKPIInfo`中移除了`presentTime`。

API 返回类型：

```typescript
interface Response<T> {
  code: number;
  message: string;
  data: T;
}
```

对于每一个 API，下文将只给出`T`的类型。

## Mapping（`./api/mapping`）

`T = APIMapping`

```typescript
interface APIMapping {
  character: Record<string, string>;
  entity: Record<string, string>;
  entityBlacklist: string[];
  entityCombine: Record<string, string>;
  missionType: Record<string, string>;
  resource: Record<string, string>;
  weapon: Record<string, string>;
  weaponCombine: Record<string, string>;
  weaponHero: Record<string, string>;
}
```

## 概览（`./api/general`）

### 任务概览(`./`)

`T = GeneralInfo`

```typescript
// “之前”与“最近”的分界线为所有有效局数 * 0.2（若所得的值小于10，则取10）.
interface DeltaData {
  prev: number; // 考虑“之前”任务的数值
  recent: number; // 考虑“最近”任务的数值
  total: number; // 考虑所有任务的数值
}

interface GeneralInfo {
  gameCount: number;
  validRate: number; // 游戏有效率
  totalMissionTime: number;
  averageMissionTime: DeltaData;
  uniquePlayerCount: number; // 遇到不同玩家的数量（按Steam名称区分）
  openRoomRate: DeltaData; // 公开房间比例（若有至少一个非好友玩家，则判断为公开房间）
  passRate: DeltaData; // 任务通过率
  averageDifficulty: DeltaData;
  averageKillNum: DeltaData;
  averageDamage: DeltaData;
  averageDeathNumPerPlayer: DeltaData;
  averageMineralsMined: DeltaData;
  averageSupplyCountPerPlayer: DeltaData;
  averageRewardCredit: DeltaData;
}
```

### 任务类型信息（`./mission_type`）

`T = MissionInfo`

```typescript
interface MissionTypeData {
  averageDifficulty: number;
  averageMissionTime: number;
  averageRewardCredit: number;
  creditPerMinute: number; // 总奖励代币数 / 总任务时间（分钟）
  missionCount: number;
  passRate: number;
}

interface MissionInfo {
  missionTypeMap: Record<string, string>; // mission_game_id -> 任务中文名称
  missionTypeData: Record<string, MissionTypeData>; // mission_game_id -> MissionTypeData
}
```

### 玩家信息（`./player`）

`T = PlayerData`

```typescript
interface PlayerData {
  averageDeathNum: number;
  averageMineralsMined: number;
  averageReviveNum: number;
  averageSupplyCount: number;
  averageSupplyEfficiency: number; // 每份补给最多回复50%弹药（不含特长），故定义为2 * 弹药比例变化量；若大于1（“补给大师”特长），仍保留
  characterInfo: Record<string, number>; // 该玩家选择每个角色的次数：character_game_id -> 选择次数
  validMissionCount: number; // 该玩家有效**游戏局数**
}

interface PlayerInfo {
  characterMap: Record<string, string>; // character_game_id -> 角色中文名
  playerData: Record<string, PlayerData>; // player_name -> PlayerInfo
  // 按之前80%游戏计算出的玩家数据
  prevPlayerData: Record<string, PlayerData>;
}
```

### 角色选择次数（`./character_info`）

`T = CharacterInfo`

```typescript
interface CharacterInfo {
  characterChoiceCount: Record<string, number>; // character_game_id -> 选择次数
  characterMapping: Record<string, string>; // character_game_id -> 角色中文名
}
```

### 角色信息（`./character`）

`T = CharacterGeneralInfo`

```typescript
interface CharacterGeneralData {
  playerIndex: number; // 有效**数据数**（采用玩家指数）
  reviveNum: number;
  deathNum: number;
  mineralsMined: number;
  supplyCount: number;
  supplyEfficiency: number;
}

interface CharacterGeneralInfo {
  characterData: Record<string, CharacterGeneralData>; // character_game_id -> CharacterGeneralData
  characterMapping: Record<string, string>; // character_game_id -> 角色中文名
}
```

## 伤害（`./api/damage`）

### 玩家伤害信息（`./`）

`T = OverallDamageInfo`

```typescript
interface PlayerDamageInfo {
  damage: Record<string, number>; // entity_game_id -> 总计受到该玩家伤害
  kill: Record<string, number>; // entity_game_id -> 该玩家总计击杀数
  ff: {
    cause: Record<string, { gameCount: number; damage: number }>; // 承受玩家 -> { gameCount: number; damage: number }
    take: Record<string, { gameCount: number; damage: number }>; // 造成玩家 -> { gameCount: number; damage: number }
  };
  averageSupplyCount: number;
  validGameCount: number; // 有效**游戏局数**
}

interface OverallDamageInfo {
  info: Record<string, PlayerDamageInfo>; // player_name -> PlayerDamageInfo
  prevInfo: Record<string, PlayerDamageInfo>; // player_name -> PlayerDamageInfo
  entityMapping: Record<string, string>; // entity_game_id -> 中文名
}
```

### 武器伤害信息（`./weapon`）

`T = Record<string, WeaponDamageInfo>` weapon_game_id -> WeaponDamageInfo

```typescript
interface WeaponDamageInfo {
  damage: number; // 总计伤害，不含友伤
  friendlyFire: number; // 总计友伤
  heroGameId: string; // 拥有该武器的角色的game_id
  mappedName: string; // 该武器的中文名
  validGameCount: number; // 有效**游戏局数**
}
```

### 角色伤害信息（`./character`）

`T = Record<string, CharacterDamageInfo>` character_game_id -> CharacterDamageInfo

```typescript
interface CharacterDamageInfo {
  damage: number; // 总计造成伤害，不含友伤
  friendlyFire: {
    cause: number; // 总计造成友伤
    take: number; // 总计受到友伤
  };
  playerIndex: number; // 有效**数据数**
  mappedName: string; // 该角色中文名
}
```

### 敌人信息（`./entity`）

`T = EntityDamageInfo`

```typescript
interface EntityDamageInfo {
  damage: Record<string, number>; // entity_game_id -> damage
  kill: Record<string, number>; // entity_game_id -> kill_num
  entityMapping: Record<string, string>; // entity_game_id -> 中文名
}
```

## 任务（`./api/mission`）

### 任务列表（`./mission_list`）

```typescript
type T = {
  missionInfo: MissionInfo[];
  missionTypeMapping: Record<string, string>; // mission_type_id -> 任务类型中文名
};
```

```typescript
interface MissionInfo {
  missionId: number;
  beginTimestamp: number; // 任务开始时间戳
  missionTime: number; // 任务进行时间
  missionTypeId: string;
  hazardId: number;
  missionResult: number; // 0 -> 已完成； 1 -> 失败； 2 -> 放弃
  rewardCredit: number; // 奖励代币数量
  missionInvalid: boolean;
  missionInvalidReason: string; // 若任务有效，则为""
}
```

### 任务信息（`./<int:mission_id>/info`）

`T = MissionGeneralInfo`

```typescript
interface MissionGeneralInfo {
  missionId: number;
  missionBeginTimestamp: number;
  missionInvalid: boolean;
  missionInvalidReason: string;
}
```

### 本任务玩家角色信息（`./<int:mission_id>/basic`）

`T = Record<string, string>` player_name -> character_game_id

### 任务概览（`./<int:mission_id>/general`）

`T = MissionGeneralData`

```typescript
interface MissionGeneralPlayerInfo {
  characterGameId: string;
  playerRank: number; // 玩家“蓝等”
  characterRank: number; // 所选角色“红等”
  characterPromotion: number; // 所选角色晋升次数
  presentTime: number; // 该玩家本任务中的游戏时间
  reviveNum: number; // 救人次数
  deathNum: number; // 倒地次数
  playerEscaped: number; // 是否在任务结束时成功撤离
}

interface MissionGeneralData {
  beginTimeStamp: number;
  hazardId: number;
  missionResult: number;
  missionTime: number;
  missionTypeId: string;
  playerInfo: Record<string, MissionGeneralPlayerInfo>; // player_name -> MissionGeneralPlayerInfo
  rewardCredit: number;
  totalDamage: number;
  totalKill: number;
  totalMinerals: number; // 总计矿石采集量
  totalNitra: number; // 总计硝石采集量
  totalSupplyCount: number;
}
```

### 任务玩家伤害统计（`./<int:mission_id>/damage`）

```typescript
type T = {
  info: Record<string, PlayerDamageInfo>; // player_name -> PlayerDamageInfo
  entityMapping: Record<string, string>; // entity_game_id -> 中文名
};
```

```typescript
interface FriendlyFireInfo {
  cause: Record<string, number>;
  take: Record<string, number>;
}

interface PlayerDamageInfo {
  damage: Record<string, number>; // entity_game_id -> 总计伤害
  kill: Record<string, number>; // entity_game_id -> 总计击杀数
  ff: FriendlyFireInfo;
  supplyCount: number; // 补给份数
}
```

### 任务武器伤害统计（`./<int:mission_id>/weapon`）

`T = Record<string, WeaponDamageInfo>` weapon_id -> WeaponDamageInfo

```typescript
interface WeaponDamageInfo {
  damage: number; // 本任务中总计造成伤害
  friendlyFire: number; // 本任务中总计造成友伤
  characterGameId: string; // 拥有该武器的角色的character_game_id
  mappedName: string; // 武器中文名
}
```

### 任务资源采集统计（`./<int:mission_id>/resource`）

```typescript
type T = {
  data: Record<string, PlayerResourceData>; // player_name -> PlayerResourceInfo
  resourceMapping: Record<string, string>; // resource_game_id -> 资源中文名
};
```

```typescript
interface PlayerResourceData {
  resource: Record<string, number>; // resource_game_id -> 本局中该玩家采集量
  supply: {
    // 玩家补给信息，每个元素为一次补给
    ammo: number; // 回复弹药量比例
    health: number; // 回复生命值比例
  }[];
}
```

### 任务玩家 KPI（`./<int:mission_id>/kpi`）

`T = MissionKPIInfo[]` 每个元素为一个角色子类型的 KPI 信息

```typescript
interface KPIComponent {
  name: string; // 该KPI组成部分的中文名
  sourceValue: number; // 该KPI组成部分的原始数值（未加权，未赋分）
  weightedValue: number; // 该KPI组成部分的加权值
  missionTotalWeightedValue: number; // 该KPI组成部分的所有玩家加权值之和
  rawIndex: number; // 未赋分、未修正的KPI项目指标
  correctedIndex: number; // 人数及角色分配修正因子修正后的KPI项目指标
  transformedIndex: number; // 赋分后的KPI项目指标
  weight: number; // 该KPI组成部分的权重
}

interface MissionKPIInfo {
  playerName: string;
  kpiCharacterType: string; // scout -> 侦察（辅助型），scout_special -> 侦察（输出型）
  weightedKill: number; // 加权击杀数
  weightedDamage: number; // 加权伤害
  priorityDamage: number; // 高威胁目标伤害
  reviveNum: number; // 救人次数
  deathNum: number; // 倒地次数
  friendlyFire: number; // 造成友伤
  nitra: number; // 硝石采集量
  supplyCount: number; // 补给份数
  weightedResource: number; // 总计资源采集量（加权）
  component: KPIComponent[]; // 每个元素为一个KPI组成部分的详细信息
  missionKPI: number; // 原始KPI
}
```

## KPI（`./api/kpi`）

### 当前 KPI 配置信息（`./`）

```typescript
type T = {
  version: string; // 当前KPI版本
};
```

### 角色权值表（`./weight_table`）

`T = WeightTableData[]` 每个元素为一种 entity 的权值信息

```typescript
interface WeightTableData {
  entityGameId: string;
  priority: number; // 高威胁目标权值
  driller: number; // 钻机权值
  gunner: number; // 枪手权值
  engineer: number; // 工程权值
  scout: number; // 辅助型侦察权值
  scoutSpecial: number; // 输出型侦察权值
}
```

### 人数及角色分配修正因子$\Gamma$（`./gamma`）

`type T = Record<string, GammaInnerInfo>` "kill", "damage", "nitra", "minerals" -> GammaInnerInfo

```typescript
type GammaInnerInfo = Record<
  string, // kpiCharacterType
  {
    playerIndex: number; // 有效**数据数**
    value: number; // 数据平均值
    ratio: number; // 修正因子
  }
>;
```

### 玩家 KPI 信息（`./player_kpi`）

```typescript
type T = Record<
  string, // player_name
  {
    playerIndex: number; // 总计玩家指数
    playerKPI: number; // 玩家KPI
    byCharacter: Record<string, PlayerCharacterKPIInfo>; // character_game_id -> PlayerCharacterKPIInfo
  }
>;
```

```typescript
interface MissionKPIInfo {
  missionId: number;
  beginTimestamp: number;
  playerIndex: number; // 该玩家在该任务中的玩家指数
  missionKPI: number; // 原始KPI
}

interface PlayerCharacterKPIInfo {
  playerIndex: number; // 该玩家在该角色上的总计玩家指数
  characterKPI: number; // 该玩家在该角色上的KPI
  characterKPIType: string;
  missionList: MissionKPIInfo[]; // 每个元素为该角色其中一次任务的KPI信息
}
```

### Bot KPI Info（`./bot_kpi_info`）

```typescript
interface PlayerBotKPIInfo {
  // (recent - prev) / prev
  deltaPercent: number;
  // 该玩家统计所有任务得出的玩家KPI
  overall: number;
  // 该玩家统计近20%任务得出的玩家KPI
  recent: number;
}
```

```typescript
type T = Record<
  string, // player_name
  PlayerBotKPIInfo
>;
```

## 信息（`./info`）

### 路人信息（`./brothers`）

`T = BrothersData`

```typescript
interface OverallBrothersInfo {
  unfamiliarPlayerCount: number;
  playerSpotPercent: number;
  playerAverageSpot: number;
  playerGeTwoPercent: number;
}

interface BrotherInfo {
  gameCount: number;
  presenceTime: number;
  lastSpot: number;
  spotCount: number;
  timestampList: number[];
}

interface BrothersData {
  overall: OverallBrothersInfo;
  player: Record<string, BrotherInfo>;
}
```

### 武器偏好信息（`./weapon_preference`）

```typescript
type T = Record<
  string, // character_game_id
  {
    0: [string, number][]; // (weapon_game_id, preference_index)
    1: [string, number][];
  }
>;
```
