<script setup lang="ts">
import { ref } from "vue";
import { NGrid, NGi, NCard, NDataTable, useMessage, type DataTableColumns } from "naive-ui";
import { translate } from "@/mapping";
import type { Response } from "@/type";
import { formatPercent } from "@/formatter";

type WeaponPreferenceData = Record<
  string,
  {
    0: [string, number][];
    1: [string, number][];
  }
>;

interface WeaponPreferenceTableRow {
  weaponAPercent: number;
  weaponBPercent: number;
  weaponCPercent: number;
}

function createWeaponPreferenceTableColumns(
  weaponPreferenceData: WeaponPreferenceData | undefined,
  characterId: string,
  weaponType: number,
): DataTableColumns<WeaponPreferenceTableRow> {
  if (weaponPreferenceData === undefined) {
    return [];
  }
  let selector: "0" | "1" = "0";
  if (weaponType === 1) {
    selector = "1";
  }
  return [
    {
      title: translate(weaponPreferenceData[characterId][selector][0][0]).value,
      key: "weaponAPercent",
      align: "center",
      render: (row) => formatPercent(row.weaponAPercent),
      width: 200,
    },
    {
      title: translate(weaponPreferenceData[characterId][selector][1][0]).value,
      key: "weaponBPercent",
      align: "center",
      render: (row) => formatPercent(row.weaponBPercent),
      width: 200,
    },
    {
      title: translate(weaponPreferenceData[characterId][selector][2][0]).value,
      key: "weaponCPercent",
      align: "center",
      render: (row) => formatPercent(row.weaponCPercent),
      width: 200,
    },
  ];
}

function generateWeaponPreferenceTableData(
  weaponPreferenceData: WeaponPreferenceData | undefined,
  characterId: string,
  weaponType: number,
): WeaponPreferenceTableRow[] {
  if (weaponPreferenceData === undefined) {
    return [];
  }
  let selector: "0" | "1" = "0";
  if (weaponType === 1) {
    selector = "1";
  }

  let totalCount = 0;
  for (const [, weaponCount] of weaponPreferenceData[characterId][selector]) {
    totalCount += weaponCount;
  }

  return [
    {
      weaponAPercent: weaponPreferenceData[characterId][selector][0][1] / totalCount,
      weaponBPercent: weaponPreferenceData[characterId][selector][1][1] / totalCount,
      weaponCPercent: weaponPreferenceData[characterId][selector][2][1] / totalCount,
    },
  ];
}

const message = useMessage();

const weaponPreferenceData = ref<WeaponPreferenceData>();

fetch("./api/info/weapon_preference")
  .then((res) => res.json())
  .then((data: Response<WeaponPreferenceData>) => {
    if (data.code !== 200) {
      message.error(`API Error while getting weapon preference info: ${data.code} ${data.message}`);
    } else {
      weaponPreferenceData.value = data.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error while getting weapon preference info: ${err}`);
  });
</script>
<template>
  <n-grid :cols="2">
    <n-gi>
      <n-card title="钻机">
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'DRILLER', 0)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'DRILLER', 0)"
          :pagination="false"
        ></n-data-table>
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'DRILLER', 1)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'DRILLER', 1)"
          :pagination="false"
        ></n-data-table>
      </n-card>
    </n-gi>
    <n-gi>
      <n-card title="枪手">
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'GUNNER', 0)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'GUNNER', 0)"
          :pagination="false"
        ></n-data-table>
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'GUNNER', 1)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'GUNNER', 1)"
          :pagination="false"
        ></n-data-table>
      </n-card>
    </n-gi>
    <n-gi>
      <n-card title="工程">
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'ENGINEER', 0)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'ENGINEER', 0)"
          :pagination="false"
        ></n-data-table>
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'ENGINEER', 1)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'ENGINEER', 1)"
          :pagination="false"
        ></n-data-table>
      </n-card>
    </n-gi>
    <n-gi>
      <n-card title="侦察">
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'SCOUT', 0)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'SCOUT', 0)"
          :pagination="false"
        ></n-data-table>
        <n-data-table
          :columns="createWeaponPreferenceTableColumns(weaponPreferenceData, 'SCOUT', 1)"
          :data="generateWeaponPreferenceTableData(weaponPreferenceData, 'SCOUT', 1)"
          :pagination="false"
        ></n-data-table>
      </n-card>
    </n-gi>
  </n-grid>
</template>
