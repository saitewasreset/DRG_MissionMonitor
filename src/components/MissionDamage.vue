<script setup lang="ts">
type PlayerBasicInfo = Record<string, string>;

interface FriendlyFireInfo {
  cause: Record<string, number>;
  take: Record<string, number>;
}

interface PlayerDamageInfo {
  damage: Record<string, number>;
  kill: Record<string, number>;
  ff: FriendlyFireInfo;
  supplyCount: number;
}

interface WeaponDamageInfo {
  damage: number;
  friendlyFire: number;
  characterGameId: string;
  mappedName: string;
}

interface PlayerDamageInfoResponse {
  code: number;
  message: string;
  data: {
    info: Record<string, PlayerDamageInfo>;
    entityMapping: Record<string, string>;
  };
}

interface WeaponDamageInfoResponse {
  code: number;
  message: string;
  data: Record<string, WeaponDamageInfo>;
}

interface DamageTableRow {
  playerName: string;
  kill: number;
  damage: number;
  regularSupplyCount: number;
  damagePerSupply: number;
}

interface WeaponTableRow {
  weaponName: string;
  damage: number;
  friendlyFire: number;
  friendlyFireRate: number;
  heroGameId: string;
}

interface FriendlyFireTableRow {
  playerName: string;
  take: number;
  cause: number;
  totalDamage: number;
  friendlyFireRate: number;
}

interface EnemyTableRow {
  enemyName: string;
  kill: number;
  damage: number;
}

import { ref, h, watch } from "vue";

import {
  useMessage,
  type DataTableColumns,
  type DataTableCreateSummary,
  NDataTable,
  NCard,
} from "naive-ui";

import { generateCharacterClass, nFormatter } from "../formatter";
import { translate } from "../mapping";

import Plotly from "plotly.js-basic-dist";
import type { Data } from "plotly.js-basic-dist";

import type { Response } from "@/type";

function createDamageTableColumns(): DataTableColumns<DamageTableRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      width: 200,
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(playerBasicInfoData.value[row.playerName]) },
          row.playerName,
        );
      },
    },
    {
      title: "击杀数",
      key: "kill",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.kill - b.kill;
      },
    },
    {
      title: "伤害",
      key: "damage",
      width: 100,
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.damage - b.damage;
      },
      render(row) {
        return nFormatter(row.damage, 2);
      },
    },
    {
      title: "约化补给数",
      key: "regularSupplyCount",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.regularSupplyCount - b.regularSupplyCount;
      },
    },
    {
      title: "每约化补给伤害",
      key: "damagePerSupply",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.damagePerSupply - b.damagePerSupply;
      },
      render(row) {
        return nFormatter(row.damagePerSupply, 2);
      },
    },
  ];
}

function generateDamageInfoTableData(
  playerDamageInfoData: Record<string, PlayerDamageInfo>,
): DamageTableRow[] {
  let result = [];
  for (const [playerName, playerData] of Object.entries(playerDamageInfoData)) {
    let totalKillCount = 0;
    let totalDamage = 0.0;

    for (const killCount of Object.values(playerData.kill)) {
      totalKillCount += killCount;
    }

    for (const damage of Object.values(playerData.damage)) {
      totalDamage += damage;
    }

    result.push({
      playerName,
      kill: totalKillCount,
      damage: totalDamage,
      regularSupplyCount: playerData.supplyCount + 1,
      damagePerSupply: totalDamage / (playerData.supplyCount + 1),
    });
  }

  return result;
}

function createDamagePlot(playerDamageInfoData: Record<string, PlayerDamageInfo>) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 2,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let killDataList: number[] = [];
  let damageDataList: number[] = [];
  let supplyCountList: number[] = [];
  let damagePerRegularSupplyList: number[] = [];

  for (const [playerName, playerData] of Object.entries(playerDamageInfoData)) {
    plotLabels.push(playerName);

    let totalKillCount = 0;
    let totalDamage = 0.0;

    for (const killCount of Object.values(playerData.kill)) {
      totalKillCount += killCount;
    }

    for (const damage of Object.values(playerData.damage)) {
      totalDamage += damage;
    }

    killDataList.push(totalKillCount);
    damageDataList.push(totalDamage);
    supplyCountList.push(playerData.supplyCount);
    damagePerRegularSupplyList.push(totalDamage / (playerData.supplyCount + 1));
  }

  let data: Data[] = [
    {
      values: killDataList,
      labels: plotLabels,
      type: "pie",
      name: "击杀数",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "击杀数",
      },
    },
    {
      values: damageDataList,
      labels: plotLabels,
      type: "pie",
      name: "伤害",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "伤害",
      },
    },
    {
      values: supplyCountList,
      labels: plotLabels,
      type: "pie",
      name: "补给数",
      domain: {
        row: 1,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "补给数",
      },
    },
    {
      values: damagePerRegularSupplyList,
      labels: plotLabels,
      type: "pie",
      name: "每约化补给伤害",
      domain: {
        row: 1,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "每约化补给伤害",
      },
    },
  ];

  Plotly.newPlot("damage-plot-container", data, plotLayout);
}

function createWeaponPlot(weaponDamageInfoData: Record<string, WeaponDamageInfo>) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 1,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let damageDataList: number[] = [];
  let friendlyFireDataList: number[] = [];

  for (const weaponData of Object.values(weaponDamageInfoData)) {
    plotLabels.push(weaponData.mappedName);
    damageDataList.push(weaponData.damage);
    friendlyFireDataList.push(weaponData.friendlyFire);
  }

  let data: Data[] = [
    {
      values: damageDataList,
      labels: plotLabels,
      type: "pie",
      name: "伤害",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "伤害",
      },
    },
    {
      values: friendlyFireDataList,
      labels: plotLabels,
      type: "pie",
      name: "友伤",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "友伤",
      },
    },
  ];

  Plotly.newPlot("weapon-plot-container", data, plotLayout);
}

function createWeaponTableColumns(): DataTableColumns<WeaponTableRow> {
  return [
    {
      title: "武器",
      key: "weaponName",
      width: 200,
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(row.heroGameId) },
          translate(row.weaponName).value,
        );
      },
    },
    {
      title: "伤害",
      key: "damage",
      defaultSortOrder: "descend",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.damage - b.damage;
      },
      render(row) {
        return nFormatter(row.damage, 2);
      },
    },
    {
      title: "友伤",
      key: "friendlyFire",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.friendlyFire - b.friendlyFire;
      },
      render(row) {
        return nFormatter(row.friendlyFire, 2);
      },
    },
    {
      title: "友伤比例",
      key: "friendlyFireRate",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.friendlyFireRate - b.friendlyFireRate;
      },
      render(row) {
        return `${(row.friendlyFireRate * 1000).toFixed(2)}‰`;
      },
    },
  ];
}

function createFriendlyFireTableColumns(): DataTableColumns<FriendlyFireTableRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      width: 200,
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(playerBasicInfoData.value[row.playerName]) },
          row.playerName,
        );
      },
    },
    {
      title: "受到友伤",
      key: "take",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.take - b.take;
      },
      render(row) {
        return nFormatter(row.take, 2);
      },
    },
    {
      title: "造成友伤",
      key: "cause",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.cause - b.cause;
      },
      render(row) {
        return nFormatter(row.cause, 2);
      },
    },
    {
      title: "总计伤害",
      key: "totalDamage",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.totalDamage - b.totalDamage;
      },
      render(row) {
        return nFormatter(row.totalDamage, 2);
      },
    },
    {
      title: "友伤比例",
      key: "friendlyFireRate",
      width: 100,
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.friendlyFireRate - b.friendlyFireRate;
      },
      render(row) {
        return `${(row.friendlyFireRate * 1000).toFixed(2)}‰`;
      },
    },
  ];
}

function generateFriendlyFireTableData(
  data: Record<string, PlayerDamageInfo>,
): FriendlyFireTableRow[] {
  let result = [];
  for (const [playerName, playerData] of Object.entries(data)) {
    let totalTake = 0.0;
    let totalCause = 0.0;

    let totalDamage = 0.0;

    for (const damage of Object.values(playerData.damage)) {
      totalDamage += damage;
    }

    for (const damage of Object.values(playerData.ff.take)) {
      totalTake += damage;
    }

    for (const damage of Object.values(playerData.ff.cause)) {
      totalCause += damage;
    }

    result.push({
      playerName,
      take: totalTake,
      cause: totalCause,
      totalDamage: totalCause + totalDamage,
      friendlyFireRate: totalCause / (totalCause + totalDamage),
    });
  }

  return result;
}

function generateWeaponTableData(
  weaponDamageInfoData: Record<string, WeaponDamageInfo>,
): WeaponTableRow[] {
  let result = [];

  for (const [weaponName, weaponData] of Object.entries(weaponDamageInfoData)) {
    result.push({
      weaponName,
      damage: weaponData.damage,
      friendlyFire: weaponData.friendlyFire,
      friendlyFireRate: weaponData.friendlyFire / (weaponData.damage + weaponData.friendlyFire),
      heroGameId: weaponData.characterGameId,
    });
  }
  return result;
}

const createDamageTableSummary: DataTableCreateSummary = (pageData) => {
  const rows = pageData as unknown as DamageTableRow[];

  let totalKill = 0;
  let totalDamage = 0.0;
  let totalRegularSupplyCount = 0;

  for (const row of rows) {
    totalKill += row.kill;
    totalDamage += row.damage;
    totalRegularSupplyCount += row.regularSupplyCount;
  }

  let totalDamagePerSupply = totalDamage / totalRegularSupplyCount;

  return {
    playerName: { value: "总计" },
    kill: { value: totalKill },
    damage: { value: nFormatter(totalDamage, 2) },
    regularSupplyCount: { value: totalRegularSupplyCount },
    damagePerSupply: { value: nFormatter(totalDamagePerSupply, 2) },
  };
};

function createFriendlyFirePlot(playerDamageInfoData: Record<string, PlayerDamageInfo>) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 1,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let takeList: number[] = [];
  let causeList: number[] = [];

  for (const [playerName, playerData] of Object.entries(playerDamageInfoData)) {
    plotLabels.push(playerName);
    let currentTotalTake = 0.0;
    let currentTotalCause = 0.0;

    for (const damage of Object.values(playerData.ff.take)) {
      currentTotalTake += damage;
    }

    for (const damage of Object.values(playerData.ff.cause)) {
      currentTotalCause += damage;
    }

    takeList.push(currentTotalTake);
    causeList.push(currentTotalCause);
  }

  let data: Data[] = [
    {
      values: takeList,
      labels: plotLabels,
      type: "pie",
      name: "受到友伤",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "受到友伤",
      },
    },
    {
      values: causeList,
      labels: plotLabels,
      type: "pie",
      name: "造成友伤",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "造成友伤",
      },
    },
  ];

  Plotly.newPlot("friendlyfire-plot-container", data, plotLayout);
}

function createEnemyTableColumns(): DataTableColumns<EnemyTableRow> {
  return [
    {
      title: "敌人",
      key: "enemyName",
      width: 200,
      align: "center",
      render(row) {
        return h("span", translate(row.enemyName).value);
      },
    },
    {
      title: "击杀数",
      key: "kill",
      width: 100,
      align: "center",
      sorter(a, b) {
        return a.kill - b.kill;
      },
    },
    {
      title: "受到伤害",
      key: "damage",
      width: 100,
      align: "center",
      defaultSortOrder: "descend",
      sorter(a, b) {
        return a.damage - b.damage;
      },
      render(row) {
        return nFormatter(row.damage, 2);
      },
    },
  ];
}

function generateEnemyTableData(
  playerDamageInfoData: Record<string, PlayerDamageInfo>,
): EnemyTableRow[] {
  let enemyMap: Record<string, EnemyTableRow> = {};
  let result = [];
  for (const playerData of Object.values(playerDamageInfoData)) {
    for (const [enemyName, killCount] of Object.entries(playerData.kill)) {
      if (enemyMap[enemyName] == undefined) {
        enemyMap[enemyName] = {
          enemyName,
          kill: 0,
          damage: 0.0,
        };
      }

      enemyMap[enemyName].kill += killCount;
    }

    for (const [enemyName, damage] of Object.entries(playerData.damage)) {
      if (enemyMap[enemyName] == undefined) {
        enemyMap[enemyName] = {
          enemyName,
          kill: 0,
          damage: 0.0,
        };
      }

      enemyMap[enemyName].damage += damage;
    }
  }

  for (const enemyData of Object.values(enemyMap)) {
    result.push(enemyData);
  }

  return result;
}

function createEnemyPlot(
  playerDamageInfoData: Record<string, PlayerDamageInfo>,
  entityMap: Record<string, string>,
) {
  const plotLayout: Partial<Plotly.Layout> = {
    width: 600,
    height: 400,
    grid: {
      rows: 1,
      columns: 2,
    },
    paper_bgcolor: "rgba(255,255,255, 0)",
    plot_bgcolor: "rgba(255,255,255, 0)",
  };

  let plotLabels: string[] = [];
  let killNumList: number[] = [];
  let damageList: number[] = [];

  let enemyMap: Record<string, EnemyTableRow> = {};

  for (const playerData of Object.values(playerDamageInfoData)) {
    for (const [enemyName, killCount] of Object.entries(playerData.kill)) {
      if (enemyMap[enemyName] == undefined) {
        enemyMap[enemyName] = {
          enemyName,
          kill: 0,
          damage: 0.0,
        };
      }

      enemyMap[enemyName].kill += killCount;
    }

    for (const [enemyName, damage] of Object.entries(playerData.damage)) {
      if (enemyMap[enemyName] == undefined) {
        enemyMap[enemyName] = {
          enemyName,
          kill: 0,
          damage: 0.0,
        };
      }

      enemyMap[enemyName].damage += damage;
    }
  }

  for (const [enemyName, enemyData] of Object.entries(enemyMap)) {
    if (entityMap[enemyName] == undefined) {
      plotLabels.push(enemyName);
    } else {
      plotLabels.push(entityMap[enemyName]);
    }
    killNumList.push(enemyData.kill);
    damageList.push(enemyData.damage);
  }

  let data: Data[] = [
    {
      values: killNumList,
      labels: plotLabels,
      type: "pie",
      name: "击杀数",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "击杀数",
      },
    },
    {
      values: damageList,
      labels: plotLabels,
      type: "pie",
      name: "受到伤害",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "受到伤害",
      },
    },
  ];

  Plotly.newPlot("enemy-plot-container", data, plotLayout);
}

const playerBasicInfoData = ref<PlayerBasicInfo>({});

const playerDamageInfoData = ref<Record<string, PlayerDamageInfo> | null>(null);

const damgeInfoTableData = ref<DamageTableRow[]>([]);

const weaponInfoTableData = ref<WeaponTableRow[]>([]);

const friendlyFireTableData = ref<FriendlyFireTableRow[]>([]);

const enemyTableData = ref<EnemyTableRow[]>([]);

const props = defineProps<{ missionId?: number }>();

const message = useMessage();

watch(
  () => props.missionId,
  () => {
    fetch(`./api/mission/${props.missionId == undefined ? 1 : props.missionId}/basic`)
      .then((res) => res.json())
      .then((data: Response<PlayerBasicInfo>) => {
        if (data.code !== 200) {
          message.error(`API Error: ${data.code} ${data.message}`);
        }
        playerBasicInfoData.value = data.data;
      })
      .catch((e) => {
        message.error(`HTTP Error: ${e}`);
      });

    fetch(`./api/mission/${props.missionId == undefined ? 1 : props.missionId}/damage`)
      .then((res) => res.json())
      .then((data: PlayerDamageInfoResponse) => {
        if (data.code !== 200) {
          message.error(`API Error: ${data.code} ${data.message}`);
        }
        playerDamageInfoData.value = data.data.info;
        damgeInfoTableData.value = generateDamageInfoTableData(data.data.info);
        createDamagePlot(data.data.info);

        friendlyFireTableData.value = generateFriendlyFireTableData(data.data.info);
        createFriendlyFirePlot(data.data.info);

        enemyTableData.value = generateEnemyTableData(data.data.info);
        createEnemyPlot(data.data.info, data.data.entityMapping);
      })
      .catch((e) => {
        message.error(`HTTP Error: ${e}`);
      });

    fetch(`./api/mission/${props.missionId == undefined ? 1 : props.missionId}/weapon`)
      .then((res) => res.json())
      .then((data: WeaponDamageInfoResponse) => {
        if (data.code !== 200) {
          message.error(`API Error: ${data.code} ${data.message}`);
        } else {
          weaponInfoTableData.value = generateWeaponTableData(data.data);
          createWeaponPlot(data.data);
        }
      })
      .catch((e) => {
        message.error(`HTTP Error: ${e}`);
      });
  },
  { immediate: true },
);
</script>

<template>
  <n-card title="输出统计">
    <div class="table-plot-container">
      <n-data-table
        :columns="createDamageTableColumns()"
        :data="damgeInfoTableData"
        :pagination="false"
        :summary="createDamageTableSummary"
        :row-key="(row: DamageTableRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
      <div id="damage-plot-container"></div>
    </div>
  </n-card>
  <n-card title="友伤统计">
    <div class="table-plot-container">
      <n-data-table
        :columns="createFriendlyFireTableColumns()"
        :data="friendlyFireTableData"
        :pagination="false"
        :row-key="(row: FriendlyFireTableRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
      <div id="friendlyfire-plot-container"></div>
    </div>
  </n-card>
  <n-card title="武器输出">
    <div class="table-plot-container">
      <n-data-table
        :columns="createWeaponTableColumns()"
        :data="weaponInfoTableData"
        :pagination="{ pageSize: 10 }"
        :row-key="(row: WeaponTableRow) => row.weaponName"
        style="width: fit-content"
      ></n-data-table>
      <div id="weapon-plot-container"></div>
    </div>
  </n-card>
  <n-card title="敌人统计">
    <div class="table-plot-container">
      <n-data-table
        :columns="createEnemyTableColumns()"
        :data="enemyTableData"
        :pagination="{ pageSize: 10 }"
        :row-key="(row: EnemyTableRow) => row.enemyName"
        style="width: fit-content"
      ></n-data-table>
      <div id="enemy-plot-container"></div>
    </div>
  </n-card>
</template>

<style scoped>
.table-plot-container {
  display: flex;
  justify-content: space-around;
  align-items: center;
}

h2 {
  text-align: center;
}
</style>
