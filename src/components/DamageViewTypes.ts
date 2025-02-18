export interface OverallDamageInfo {
  info: Record<string, PlayerDamageInfo>;
  prevInfo: Record<string, PlayerDamageInfo>;
  entityMapping: Record<string, string>;
}

export interface PlayerDamageInfo {
  damage: Record<string, number>;
  kill: Record<string, number>;
  ff: {
    cause: Record<string, { gameCount: number; damage: number }>;
    take: Record<string, { gameCount: number; damage: number }>;
  };
  averageSupplyCount: number;
  validGameCount: number;
}

export interface WeaponDamageInfo {
  damage: number;
  friendlyFire: number;
  heroGameId: string;
  mappedName: string;
  validGameCount: number;
}

export interface CharacterDamageInfo {
  damage: number;
  friendlyFire: {
    cause: number;
    take: number;
  };
  playerIndex: number;
  mappedName: string;
}

export interface EntityData {
  damage: Record<string, number>;
  kill: Record<string, number>;
  entityMapping: Record<string, string>;
}

export interface ResponseData<T> {
  code: number;
  message: string;
  data: T;
}
