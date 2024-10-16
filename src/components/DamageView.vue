<script setup lang="ts">
import { ref } from "vue";
import { useMessage, NTabs, NTabPane, NCard } from "naive-ui";

import type {
  OverallDamageInfo,
  WeaponDamageInfo,
  CharacterDamageInfo,
  EntityData,
  ResponseData,
} from "./DamageViewTypes";

import DamageDamage from "./DamageDamage.vue";
import DamageFriendlyFire from "./DamageFriendlyFire.vue";

const message = useMessage();
const overallDamageInfo = ref<OverallDamageInfo>();
const weaponDamageInfo = ref<Record<string, WeaponDamageInfo>>();
const characterDamageInfo = ref<Record<string, CharacterDamageInfo>>();
const entityData = ref<EntityData>();

fetch("./api/damage/")
  .then((res) => res.json())
  .then((data: ResponseData<OverallDamageInfo>) => {
    if (data.code !== 200) {
      message.error(`API Error while getting overall damage info: ${data.code} ${data.message}`);
    } else {
      overallDamageInfo.value = data.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error while getting overall damage info: ${err}`);
  });

fetch("./api/damage/weapon")
  .then((res) => res.json())
  .then((data: ResponseData<Record<string, WeaponDamageInfo>>) => {
    if (data.code !== 200) {
      message.error(`API Error while getting weapon damage info: ${data.code} ${data.message}`);
    } else {
      weaponDamageInfo.value = data.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error while getting weapon damage info: ${err}`);
  });

fetch("./api/damage/character")
  .then((res) => res.json())
  .then((data: ResponseData<Record<string, CharacterDamageInfo>>) => {
    if (data.code !== 200) {
      message.error(`API Error while getting character damage info: ${data.code} ${data.message}`);
    } else {
      characterDamageInfo.value = data.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error while getting character damage info: ${err}`);
  });
fetch("./api/damage/entity")
  .then((res) => res.json())
  .then((data: ResponseData<EntityData>) => {
    if (data.code !== 200) {
      message.error(`API Error while getting character damage info: ${data.code} ${data.message}`);
    } else {
      entityData.value = data.data;
    }
  })
  .catch((err) => {
    message.error(`HTTP Error while getting character damage info: ${err}`);
  });
</script>
<template>
  <n-card>
    <n-tabs type="line" default-value="damage">
      <n-tab-pane name="damage" tab="伤害" display-directive="show">
        <DamageDamage
          :overallDamageInfo="overallDamageInfo"
          :weaponDamageInfo="weaponDamageInfo"
          :characterDamageInfo="characterDamageInfo"
          :entity-data="entityData"
        ></DamageDamage>
      </n-tab-pane>
      <n-tab-pane name="friendly-fire" tab="友伤" display-directive="show">
        <DamageFriendlyFire :overallDamageInfo="overallDamageInfo"></DamageFriendlyFire>
      </n-tab-pane>
    </n-tabs>
  </n-card>
</template>
