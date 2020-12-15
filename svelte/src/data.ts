interface State {
  version: string;
  selected_game: string;
  games: { [id: string]: SetupGame };
  bep_in_ex?: BepInExInfo,
}

interface BepInExInfo {
    version?: string,
    enabled: boolean,
}

interface Game {
  id: string;
  name: string;
}

interface SetupGame {
  game: Game;
  install_path: string;
  mods: InstalledMod[];
}

interface InstalledMod {
  m: Mod;
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
}

interface Metadata {
  version: string;
  update: boolean;
  games: Map<string, Game>;
  game_mods: Map<string, Map<string, Mod>>;
  mods: Map<string, Mod>;
}
