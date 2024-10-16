<script setup lang="ts">
import { ref, watch, h } from "vue";
import { NDataTable, NCard, type DataTableColumns, type DataTableCreateSummary } from "naive-ui";
import Plotly, { type Data } from "plotly.js-basic-dist";
import type {
  OverallDamageInfo,
  WeaponDamageInfo,
  CharacterDamageInfo,
  EntityData,
} from "./DamageViewTypes";

import { nFormatter, generateCharacterClass } from "@/formatter";
import { translate } from "@/mapping";

import { getFirstGroupInfo } from "@/tool";
import DeltaView from "./DeltaView.vue";

interface OverallTableRow {
  playerName: string;
  validGameCount: number;
  averageKillNum: number[];
  averageDamage: number[];
  averageRegularSupplyCount: number[];
  averageDamagePerRegularSupply: number[];
}

interface WeaponTableRow {
  weaponName: string;
  weaponCharacter: string;
  validGameCount: number;
  averageDamage: number;
  averageFriendlyFire: number;
  friendlyFireRate: number;
}

interface CharacterTableRow {
  characterGameId: string;
  validGameCount: number;
  averageDamage: number;
  averageCauseDamage: number;
  averageTakeDamage: number;
  friendlyFireRate: number;
  heartbrokenRate: number;
}

interface EnemyTableRow {
  mappedEnemyName: string;
  totalKill: number;
  totalDamage: number;
}

const props = defineProps<{
  overallDamageInfo?: OverallDamageInfo;
  weaponDamageInfo?: Record<string, WeaponDamageInfo>;
  characterDamageInfo?: Record<string, CharacterDamageInfo>;
  entityData?: EntityData;
}>();

const overallTableData = ref<OverallTableRow[]>([]);
const weaponTableData = ref<WeaponTableRow[]>([]);
const characterTableData = ref<CharacterTableRow[]>([]);
const enemyTableData = ref<EnemyTableRow[]>([]);

const createOverallTableSummary: DataTableCreateSummary = (pageData) => {
  const rows = pageData as unknown as OverallTableRow[];

  let totalKill = 0;
  let totalDamage = 0.0;
  let totalRegularSupplyCount = 0;

  let totalPrevKill = 0.0;
  let totalPrevDamage = 0.0;
  let totalPrevRegularSupplyCount = 0;

  for (const row of rows) {
    totalKill += row.averageKillNum[1];
    totalPrevKill += row.averageKillNum[0];
    totalDamage += row.averageDamage[1];
    totalPrevDamage += row.averageDamage[0];
    totalRegularSupplyCount += row.averageRegularSupplyCount[1];
    totalPrevRegularSupplyCount += row.averageRegularSupplyCount[0];
  }

  let totalDamagePerSupply = totalDamage / totalRegularSupplyCount;
  let totalPrevDamagePerSupply = totalPrevDamage / totalPrevRegularSupplyCount;

  return {
    playerName: { value: "总计" },
    validGameCount: { value: "-" },
    averageKillNum: { value: h(DeltaView, {
        label: "",
        delta: {prev: totalPrevKill, total: totalKill},
        formatter: (x) => x.toFixed(0),
        style: "font-size: 1rem",
      }) },
    averageDamage: { value: h(DeltaView, {
        label: "",
        delta: {prev: totalPrevDamage, total: totalDamage},
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }) },
    averageRegularSupplyCount: { value: h(DeltaView, {
        label: "",
        delta: {prev: totalPrevRegularSupplyCount, total: totalRegularSupplyCount},
        formatter: (x) => x.toFixed(1),
        reverse: true,
        style: "font-size: 1rem",
      }) },
    averageDamagePerRegularSupply: { value: h(DeltaView, {
        label: "",
        delta: {prev: totalPrevDamagePerSupply, total: totalDamagePerSupply},
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }) },
  };
};

function createOverallTableColumns(): DataTableColumns<OverallTableRow> {
  return [
    {
      title: "玩家",
      key: "playerName",
      align: "center",
    },
    {
      title: "有效局数",
      key: "validGameCount",
      align: "center",
      sorter: (a, b) => a.validGameCount - b.validGameCount,
    },
    {
      title: "平均击杀数",
      key: "averageKillNum",
      align: "center",
      sorter: (a, b) => a.averageKillNum[1] - b.averageKillNum[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageKillNum[0], total: row.averageKillNum[1]},
        formatter: (x) => x.toFixed(0),
        style: "font-size: 1rem",
      }),
    },
    {
      title: "平均伤害",
      key: "averageDamage",
      align: "center",
      sorter: (a, b) => a.averageDamage[1] - b.averageDamage[1],
      defaultSortOrder: "descend",
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageDamage[0], total: row.averageDamage[1]},
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }),
    },
    {
      title: "平均约化补给数",
      key: "averageRegularSupplyCount",
      align: "center",
      sorter: (a, b) => a.averageRegularSupplyCount[1] - b.averageRegularSupplyCount[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageRegularSupplyCount[0], total: row.averageRegularSupplyCount[1]},
        reverse: true,
        formatter: (x) => x.toFixed(1),
        style: "font-size: 1rem",
      }),
    },
    {
      title: "每约化补给伤害",
      key: "averageDamagePerRegularSupply",
      align: "center",
      sorter: (a, b) => a.averageDamagePerRegularSupply[1] - b.averageDamagePerRegularSupply[1],
      render: (row) => h(DeltaView, {
        label: "",
        delta: {prev: row.averageDamagePerRegularSupply[0], total: row.averageDamagePerRegularSupply[1]},
        formatter: (x) => nFormatter(x, 2),
        style: "font-size: 1rem",
      }),
    },
  ];
}

function generateOverallTableData(): OverallTableRow[] {
  let result = [];
  if (props.overallDamageInfo === undefined) {
    return [];
  }
  for (const [playerName, playerData] of Object.entries(props.overallDamageInfo.info)) {
    let totalDamage = 0.0;
    let totalKill = 0.0;

    let prevDamage = 0.0;
    let prevKill = 0.0;

    for (const damage of Object.values(playerData.damage)) {
      totalDamage += damage;
    }

    for (const kill of Object.values(playerData.kill)) {
      totalKill += kill;
    }

    let playerPrevData = props.overallDamageInfo.prevInfo[playerName];

    for (const damage of Object.values(playerPrevData.damage)) {
      prevDamage += damage;
    }

    for (const kill of Object.values(playerPrevData.kill)) {
      prevKill += kill;
    }

    result.push({
      playerName,
      validGameCount: playerData.validGameCount,
      averageKillNum: [prevKill / playerPrevData.validGameCount, totalKill / playerData.validGameCount],
      averageDamage: [prevDamage / playerPrevData.validGameCount, totalDamage / playerData.validGameCount, ],
      averageRegularSupplyCount: [playerPrevData.averageSupplyCount + 1, playerData.averageSupplyCount + 1, ],
      averageDamagePerRegularSupply:
        [prevDamage / ((playerPrevData.averageSupplyCount + 1) * playerPrevData.validGameCount), 
        totalDamage / ((playerData.averageSupplyCount + 1) * playerData.validGameCount)  
        ],
    });
  }

  return result;
}

function createDamagePlot() {
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

  if (props.overallDamageInfo === undefined) {
    return;
  }

  for (const [playerName, playerData] of Object.entries(props.overallDamageInfo.info)) {
    plotLabels.push(playerName);

    let totalKillCount = 0;
    let totalDamage = 0.0;

    for (const killCount of Object.values(playerData.kill)) {
      totalKillCount += killCount;
    }

    for (const damage of Object.values(playerData.damage)) {
      totalDamage += damage;
    }

    killDataList.push(totalKillCount / playerData.validGameCount);
    damageDataList.push(totalDamage / playerData.validGameCount);
    supplyCountList.push(playerData.averageSupplyCount);
    damagePerRegularSupplyList.push(
      totalDamage / ((playerData.averageSupplyCount + 1) * playerData.validGameCount),
    );
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

  Plotly.newPlot("overall-plot-container", data, plotLayout);
}

function createWeaponTableColumns(): DataTableColumns<WeaponTableRow> {
  return [
    {
      title: "武器",
      key: "weaponName",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(row.weaponCharacter) },
          translate(row.weaponName).value,
        );
      },
    },
    {
      title: "有效局数",
      key: "validGameCount",
      align: "center",
      sorter: (a, b) => a.validGameCount - b.validGameCount,
    },
    {
      title: "平均伤害",
      key: "averageDamage",
      align: "center",
      sorter: (a, b) => a.averageDamage - b.averageDamage,
      defaultSortOrder: "descend",
      render(row) {
        return nFormatter(row.averageDamage, 2);
      },
    },
    {
      title: "平均友伤",
      key: "averageFriendlyFire",
      align: "center",
      sorter: (a, b) => a.averageFriendlyFire - b.averageFriendlyFire,
      render(row) {
        return nFormatter(row.averageFriendlyFire, 2);
      },
    },
    {
      title: "友伤比例",
      key: "friendlyFireRate",
      align: "center",
      sorter: (a, b) => a.friendlyFireRate - b.friendlyFireRate,
      render(row) {
        return (row.friendlyFireRate * 1000).toFixed(2) + "‰";
      },
    },
  ];
}

function createWeaponTableData(): WeaponTableRow[] {
  if (props.weaponDamageInfo === undefined) {
    return [];
  }
  let result = [];
  for (const [weaponName, weaponData] of Object.entries(props.weaponDamageInfo)) {
    result.push({
      weaponName,
      weaponCharacter: weaponData.heroGameId,
      validGameCount: weaponData.validGameCount,
      averageDamage: weaponData.damage / weaponData.validGameCount,
      averageFriendlyFire: weaponData.friendlyFire / weaponData.validGameCount,
      friendlyFireRate: weaponData.friendlyFire / (weaponData.damage + weaponData.friendlyFire),
    });
  }

  return result;
}

function createWeaponPlot() {
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

  if (props.weaponDamageInfo === undefined) {
    return;
  }

  let weaponDataList: WeaponDamageInfo[] = [];
  for (const weaponData of Object.values(props.weaponDamageInfo)) {
    weaponDataList.push(weaponData);
  }

  let damageGroupInfo = getFirstGroupInfo(
    weaponDataList,
    (a, b) => b.damage / b.validGameCount - a.damage / a.validGameCount,
    (a) => a.damage / a.validGameCount,
    (a) => a.mappedName,
    0.1,
  );

  let friendlyFireGroupInfo = getFirstGroupInfo(
    weaponDataList,
    (a, b) => b.friendlyFire / b.validGameCount - a.friendlyFire / a.validGameCount,
    (a) => a.friendlyFire / a.validGameCount,
    (a) => a.mappedName,
    0.1,
  );

  let data: Data[] = [
    {
      values: damageGroupInfo.dataList,
      labels: damageGroupInfo.labelList,
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
      values: friendlyFireGroupInfo.dataList,
      labels: friendlyFireGroupInfo.labelList,
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

function createCharacterTableColumns(): DataTableColumns<CharacterTableRow> {
  return [
    {
      title: "角色",
      key: "characterGameId",
      align: "center",
      render(row) {
        return h(
          "span",
          { class: generateCharacterClass(row.characterGameId) },
          translate(row.characterGameId).value,
        );
      },
    },
    {
      title: "有效数据数",
      key: "validGameCount",
      align: "center",
      render(row) {
        return row.validGameCount.toFixed(2);
      },
    },
    {
      title: "平均伤害",
      key: "averageDamage",
      align: "center",
      sorter: (a, b) => a.averageDamage - b.averageDamage,
      render(row) {
        return nFormatter(row.averageDamage, 2);
      },
    },
    {
      title: "平均造成友伤",
      key: "averageCauseDamage",
      align: "center",
      sorter: (a, b) => a.averageCauseDamage - b.averageCauseDamage,
      render(row) {
        return nFormatter(row.averageCauseDamage, 2);
      },
    },
    {
      title: "平均受到友伤",
      key: "averageTakeDamage",
      align: "center",
      sorter: (a, b) => a.averageTakeDamage - b.averageTakeDamage,
      render(row) {
        return nFormatter(row.averageTakeDamage, 2);
      },
    },
    {
      title: "友伤比例",
      key: "friendlyFireRate",
      align: "center",
      sorter: (a, b) => a.friendlyFireRate - b.friendlyFireRate,
      render(row) {
        return (row.friendlyFireRate * 1000).toFixed(2) + "‰";
      },
    },
    {
      title: "心碎值",
      key: "heartbrokenRate",
      align: "center",
      sorter: (a, b) => a.heartbrokenRate - b.heartbrokenRate,
      defaultSortOrder: "descend",
      render(row) {
        return row.heartbrokenRate.toFixed(2);
      },
    },
  ];
}

function generateCharacterTableData(): CharacterTableRow[] {
  if (props.characterDamageInfo === undefined) {
    return [];
  }

  let result = [];

  for (const [characterGameId, characterData] of Object.entries(props.characterDamageInfo)) {
    result.push({
      characterGameId,
      validGameCount: characterData.playerIndex,
      averageDamage: characterData.damage / characterData.playerIndex,
      averageCauseDamage: characterData.friendlyFire.cause / characterData.playerIndex,
      averageTakeDamage: characterData.friendlyFire.take / characterData.playerIndex,
      friendlyFireRate:
        characterData.friendlyFire.cause /
        (characterData.damage + characterData.friendlyFire.cause),
      heartbrokenRate: characterData.friendlyFire.take / characterData.friendlyFire.cause,
    });
  }

  return result;
}

function createCharacterPlot() {
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

  const characterToColor: Record<string, string> = {
    SCOUT: "#00BFFF",
    DRILLER: "#DAA520",
    ENGINEER: "#FF0000",
    GUNNER: "#008000",
  };

  let characterColorList: string[] = [];

  let plotLabels: string[] = [];
  let averageDamageList: number[] = [];
  let averageCauseList: number[] = [];
  let averageTakeList: number[] = [];
  let heartbrokenRateList: number[] = [];

  if (props.characterDamageInfo === undefined) {
    return;
  }

  for (const [characterGameId, characterData] of Object.entries(props.characterDamageInfo)) {
    plotLabels.push(characterData.mappedName);
    characterColorList.push(characterToColor[characterGameId]);
    averageDamageList.push(characterData.damage / characterData.playerIndex);
    averageCauseList.push(characterData.friendlyFire.cause / characterData.playerIndex);
    averageTakeList.push(characterData.friendlyFire.take / characterData.playerIndex);
    heartbrokenRateList.push(characterData.friendlyFire.take / characterData.friendlyFire.cause);
  }

  let data: Data[] = [
    {
      values: averageDamageList,
      labels: plotLabels,
      type: "pie",
      name: "平均伤害",
      domain: {
        row: 0,
        column: 1,
      },
      marker: {
        colors: characterColorList,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均伤害",
        position: "bottom center",
      },
    },
    {
      values: averageCauseList,
      labels: plotLabels,
      type: "pie",
      name: "平均造成友伤",
      domain: {
        row: 1,
        column: 0,
      },
      marker: {
        colors: characterColorList,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均造成友伤",
        position: "bottom center",
      },
    },
    {
      values: averageTakeList,
      labels: plotLabels,
      type: "pie",
      name: "平均受到友伤",
      domain: {
        row: 1,
        column: 1,
      },
      marker: {
        colors: characterColorList,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "平均受到友伤",
        position: "bottom center",
      },
    },
    {
      values: heartbrokenRateList,
      labels: plotLabels,
      type: "bar",
      name: "心碎值",
      x: heartbrokenRateList,
      y: plotLabels,
      orientation: "h",
    },
  ];

  Plotly.newPlot("character-plot-container", data, plotLayout);
}

function createEnemyTableColumns(): DataTableColumns<EnemyTableRow> {
  return [
    {
      title: "敌人",
      key: "mappedEnemyName",
      align: "center",
      render(row) {
        return h("span", row.mappedEnemyName);
      },
    },
    {
      title: "总击杀数",
      key: "totalKill",
      align: "center",
      sorter: (a, b) => a.totalKill - b.totalKill,
      render(row) {
        return nFormatter(row.totalKill, 2);
      },
    },
    {
      title: "总受到伤害",
      key: "totalDamage",
      align: "center",
      sorter: (a, b) => a.totalDamage - b.totalDamage,
      defaultSortOrder: "descend",
      render(row) {
        return nFormatter(row.totalDamage, 2);
      },
    },
  ];
}

function generateEnemyTableData(): EnemyTableRow[] {
  if (props.entityData === undefined) {
    return [];
  }

  let result = [];

  let enemyDamageMap: Record<string, number> = props.entityData.damage;
  let enemyKillMap: Record<string, number> = props.entityData.kill;

  for (const [entityGameId, totalKill] of Object.entries(enemyKillMap)) {
    result.push({
      mappedEnemyName:
        props.entityData.entityMapping[entityGameId] === undefined
          ? entityGameId
          : props.entityData.entityMapping[entityGameId],
      totalKill: totalKill,
      totalDamage: enemyDamageMap[entityGameId],
    });
  }

  return result;
}

function createEnemyPlot() {
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

  if (props.entityData === undefined) {
    return;
  }

  let enemyDamageMap: Record<string, number> = props.entityData.damage;
  let enemyKillMap: Record<string, number> = props.entityData.kill;

  let enemyDataList = [];

  for (const [entityGameId, totalKill] of Object.entries(enemyKillMap)) {
    enemyDataList.push({
      mappedName:
        props.entityData.entityMapping[entityGameId] === undefined
          ? entityGameId
          : props.entityData.entityMapping[entityGameId],
      totalKill,
      totalDamage: enemyDamageMap[entityGameId],
    });
  }

  let enemyKillGroupInfo = getFirstGroupInfo(
    enemyDataList,
    (a, b) => b.totalKill - a.totalKill,
    (a) => a.totalKill,
    (a) => a.mappedName,
    0.1,
  );

  let enemyDamageGroupInfo = getFirstGroupInfo(
    enemyDataList,
    (a, b) => b.totalDamage - a.totalDamage,
    (a) => a.totalDamage,
    (a) => a.mappedName,
    0.1,
  );

  let data: Data[] = [
    {
      values: enemyKillGroupInfo.dataList,
      labels: enemyKillGroupInfo.labelList,
      type: "pie",
      name: "总击杀数",
      domain: {
        row: 0,
        column: 0,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "总击杀数",
      },
    },
    {
      values: enemyDamageGroupInfo.dataList,
      labels: enemyDamageGroupInfo.labelList,
      type: "pie",
      name: "总伤害",
      domain: {
        row: 0,
        column: 1,
      },
      hoverinfo: "label+value+percent",
      title: {
        text: "总伤害",
      },
    },
  ];

  Plotly.newPlot("enemy-plot-container", data, plotLayout);
}

watch(
  () => props.overallDamageInfo,
  () => {
    overallTableData.value = generateOverallTableData();
    createDamagePlot();
  },
);

watch(
  () => props.weaponDamageInfo,
  () => {
    weaponTableData.value = createWeaponTableData();
    createWeaponPlot();
  },
);

watch(
  () => props.characterDamageInfo,
  () => {
    characterTableData.value = generateCharacterTableData();
    createCharacterPlot();
  },
);

watch(
  () => props.entityData,
  () => {
    enemyTableData.value = generateEnemyTableData();
    createEnemyPlot();
  },
);
</script>
<template>
  <n-card title="按玩家">
    <div class="table-plot-container">
      <n-data-table
        :columns="createOverallTableColumns()"
        :data="overallTableData"
        :pagination="false"
        :summary="createOverallTableSummary"
        :row-key="(row: OverallTableRow) => row.playerName"
        style="width: fit-content"
      ></n-data-table>
      <div id="overall-plot-container"></div></div
  ></n-card>
  <n-card title="按武器">
    <div class="table-plot-container">
      <n-data-table
        :columns="createWeaponTableColumns()"
        :data="weaponTableData"
        :pagination="{ pageSize: 10 }"
        :row-key="(row: WeaponTableRow) => row.weaponName"
        style="width: fit-content"
      ></n-data-table>
      <div id="weapon-plot-container"></div>
    </div>
  </n-card>
  <n-card title="按角色">
    <div class="table-plot-container">
      <n-data-table
        :columns="createCharacterTableColumns()"
        :data="characterTableData"
        :pagination="false"
        :row-key="(row: CharacterTableRow) => row.characterGameId"
        style="width: fit-content"
      ></n-data-table>
      <div id="character-plot-container"></div>
    </div>
  </n-card>
  <n-card title="按敌人">
    <div class="table-plot-container">
      <n-data-table
        :columns="createEnemyTableColumns()"
        :data="enemyTableData"
        :pagination="{ pageSize: 10 }"
        :row-key="(row: EnemyTableRow) => row.mappedEnemyName"
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
</style>
