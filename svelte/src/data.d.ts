interface State {
  version: string;
  selected_game: string;
  games: { [id: string]: SetupGame };
  os: string;
}

interface Game {
  id: string;
  name: string;
  recommended_mods: string[];
}

interface SetupGame {
  game: Game;
  install_path: string;
  mods: { [id: string]: InstalledMod };
  bep_in_ex: BepInExInfo;
}

interface BepInExInfo {
  version?: string;
  enabled: boolean;
}

interface InstalledMod {
  m: Mod;
  version: string;
  enabled: boolean;
}

interface Mod {
  id: string;
  name: string;
  download: string;
  version: string;
  source?: string;
  homepage?: string;
  description?: string;
  ytinu_version?: string;
  dev_mod?: boolean;
}

interface Metadata {
  version: string;
  update: boolean;
  games: { [id: string]: Game };
  game_mods: { [id: string]: { [id: string]: Mod } };
  mods: { [id: string]: Mod };
}
