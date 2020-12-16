<script lang="ts">
	import AddGameDialog from "./dialogs/AddGameDialog.svelte";
	import AboutDialog from "./dialogs/AboutDialog.svelte";
	import SettingsDialog from "./dialogs/SettingsDialog.svelte";
	import { API_BASE } from "./config";
	import ManageGameDialog from "./dialogs/ManageGameDialog.svelte";
	import ModEntry from "./ModEntry.svelte";
	import { fade } from "svelte/transition";

	let version: string = null;
	let games: Map<string, SetupGame> = null;
	let meta: Metadata = null;
	let selectedGameId = null;
	let selectedGame: SetupGame = null;
	let os = null;
	let settings: {
		dark_mode?: "system" | "dark" | "light";
		show_dev_mods?: boolean;
	} = {};
	let installed_mods: InstalledMod[] = [];
	let recommended_mods: Mod[] = [];
	let available_mods: Mod[] = [];

	let showAddGameDialog = false;
	let showManageGameDialog = false;
	let showAboutDialog = false;
	let showSettingsDialog = false;

	let expandedMod = null;

	fetchState();
	fetchMetadata();
	loadSettings();

	function updateModList() {
		if (!selectedGame || !meta) return;
		let installed = [];
		for (const mod of Object.values(selectedGame.mods)) {
			installed.push(mod);
		}
		installed.sort((a, b) => a.m.name.localeCompare(b.m.name));
		installed_mods = installed;

		let recommended = [];
		let available = [];
		for (const mod of Object.values(meta.mods)) {
			if (selectedGame.mods[mod.id]) continue;
			if (selectedGame.game.recommended_mods.indexOf(mod.id) >= 0)
				recommended.push(mod);
			else available.push(mod);
		}
		let game_mods = meta.game_mods[selectedGameId];
		if (game_mods) {
			for (const mod of Object.values(game_mods)) {
				if (selectedGame.mods[mod.id]) continue;
				if (selectedGame.game.recommended_mods.indexOf(mod.id) >= 0)
					recommended.push(mod);
				else available.push(mod);
			}
		}
		available.sort((a, b) => a.name.localeCompare(b.name));
		recommended.sort((a, b) => a.name.localeCompare(b.name));
		available_mods = available;
		recommended_mods = recommended;
	}

	function fetchState() {
		fetch(API_BASE + "state")
			.then((r) => r.json())
			.then((r) => {
				if (r === null) return;
				else if (r.error !== undefined) {
					console.error(r.error);
					return;
				}
				let state = r as State;
				os = state.os;
				version = state.version;
				games = new Map(Object.entries(state.games));
				selectedGameId = state.selected_game;
				selectedGame = selectedGameId && games.get(selectedGameId);
				updateModList();
			});
	}

	function fetchMetadata() {
		fetch(API_BASE + "metadata")
			.then((r) => r.json())
			.then((r) => {
				if (r === null) {
					meta = null;
					return;
				} else if (r.error !== undefined) {
					console.error(r.error);
					return;
				}
				meta = r as Metadata;
				updateModList();
				// TODO: Use meta.update
			});
	}

	function loadSettings() {
		settings = JSON.parse(localStorage.getItem("settings")) || {};

		if (
			settings?.dark_mode === "dark" ||
			(settings?.dark_mode !== "light" &&
				window.matchMedia?.("(prefers-color-scheme: dark)").matches)
		) {
			document.documentElement.setAttribute("data-theme", "dark");
		} else {
			document.documentElement.setAttribute("data-theme", "light");
		}
	}

	function handleClickInstall() {
		fetch(API_BASE + "toggle_modloader_installed").then(() => fetchState());
	}

	function handleClickEnable() {
		fetch(API_BASE + "toggle_modloader_enabled").then(() => fetchState());
	}

	function installMod(id: string) {
		fetch(API_BASE + "install_mod/" + id).then(() => fetchState());
	}

	function updateMod(id: string) {
		fetch(API_BASE + "update_mod/" + id).then(() => fetchState());
	}

	// function enableMod(id: string) {
	// 	fetch(API_BASE + "toggle_mod_enabled/" + id).then(() => fetchState());
	// }

	function uninstallMod(id: string) {
		fetch(API_BASE + "remove_mod/" + id).then(() => fetchState());
	}
</script>

<style lang="scss">
	:global {
		@import "./global.scss";
	}

	@import "./App.scss";
</style>

<div class="page-container">
	<div class="menubar">
		<button type="button" on:click={() => (showSettingsDialog = true)}>
			Settings
		</button>
		<button type="button" on:click={() => (showAboutDialog = true)}>
			About
		</button>
	</div>

	{#if !version || !meta}
		<div class="spinner-container" transition:fade>
			<div class="lds-hourglass" />
		</div>
	{:else}
		<div class="game-selection" transition:fade>
			<b class="game-selection--label">Game:</b>
			<select
				class="game-selection--select"
				value={selectedGameId}
				disabled={games === null || games.size < 2}>
				{#if games?.size > 0}
					{#each [...games.values()].sort() as { game: { id, name } }}
						<option value={id}>{name}</option>
					{/each}
				{:else}
					<option>
						No game setup. Use the button on the right to add one
						--&gt;
					</option>
				{/if}
			</select>
			<button
				type="button"
				disabled={games?.size > 0}
				on:click={() => (showAddGameDialog = true)}>
				Add Game
			</button>
			<button
				type="button"
				disabled={games === null || games.size == 0}
				on:click={() => (showManageGameDialog = true)}>Manage Games</button>
		</div>

		{#if selectedGame}
			<div class="sidebar" transition:fade>
				<h4>Status</h4>
				{#if selectedGame.bep_in_ex == null}
					<p>Mod Loader not installed</p>
					<p>Use the button below to install it</p>
				{:else}
					{#if selectedGame.bep_in_ex.enabled}
						<p>Mod Loader installed and enabled</p>
					{:else}
						<p>Mod Loader installed but <b>disabled</b></p>
					{/if}
					<p>
						BepInEx
						{selectedGame.bep_in_ex.version ? 'v' + selectedGame.bep_in_ex.version : '- Unknown Version'}
					</p>
				{/if}

				<div class="big-buttons">
					<button
						type="button"
						title={'Install the BepInEx Mod Loader for ' + selectedGame.game.name}
						on:click={handleClickInstall}>
						{selectedGame.bep_in_ex == null ? 'Install' : 'Uninstall'}
					</button>
					<button
						type="button"
						disabled={selectedGame.bep_in_ex == null}
						title={selectedGame.bep_in_ex?.enabled ? 'Disable The BepInEx Mod Loader without touching any mods or configuration. This will make the game launch completely cleanly.' : 'Enable the BepInEx Mod Loader'}
						on:click={handleClickEnable}>
						{selectedGame.bep_in_ex?.enabled ? 'Disable' : 'Enable'}
					</button>
				</div>

				{#if os === 'windows'}
					<div class="open-directory-buttons">
						<p>Open Directory</p>
						<div>
							<button
								type="button"
								on:click={() => fetch(API_BASE + 'open/game')}>Game</button>
							<button
								type="button"
								disabled={!selectedGame.bep_in_ex}
								on:click={() => fetch(API_BASE + 'open/mods')}>Mods</button>
							<button
								type="button"
								disabled={!selectedGame.bep_in_ex}
								on:click={() => fetch(API_BASE + 'open/configs')}>Configs</button>
						</div>
					</div>
				{/if}
			</div>

			<div class="main" transition:fade>
				{#if selectedGame.bep_in_ex}
					<div transition:fade>
						<h4>
							Mods
							<small>(Click on a Mod for more information)</small>
						</h4>
						<div class="mod-list">
							{#if installed_mods.length > 0}
								<h5>Installed</h5>
								{#each installed_mods as installed_mod (installed_mod.m.id)}
									<ModEntry
										bind:expandedMod
										onUpdate={updateMod}
										onUninstall={uninstallMod}
										{installed_mod} />
								{/each}
							{/if}
							{#if recommended_mods.length > 0}
								<h5>Recommended</h5>
								{#each recommended_mods as available_mod (available_mod.id)}
									{#if !available_mod.dev_mod || settings.show_dev_mods}
										<ModEntry
											bind:expandedMod
											onInstall={installMod}
											{available_mod} />
									{/if}
								{/each}
							{/if}
							{#if available_mods.length > 0}
								<h5>Available</h5>
								{#each available_mods as available_mod (available_mod.id)}
									{#if !available_mod.dev_mod || settings.show_dev_mods}
										<ModEntry
											bind:expandedMod
											onInstall={installMod}
											{available_mod} />
									{/if}
								{/each}
							{/if}
						</div>
					</div>
				{:else}
					<div class="not-installed" transition:fade>
						<h2>The BepInEx Mod Loader is not installed</h2>
						<h3>Use the button on the left to install it</h3>
					</div>
				{/if}
			</div>
		{:else}
			<h2 class="main--no-game" transition:fade>
				No game setup. Use the "Add Game" button above to add one.
			</h2>
		{/if}
	{/if}
</div>

{#if showManageGameDialog}
	<ManageGameDialog
		install_path={selectedGame.install_path}
		onClose={(change) => {
			showManageGameDialog = false;
			if (change) fetchState();
		}} />
{/if}

{#if showAddGameDialog}
	<AddGameDialog
		onClose={(change) => {
			showAddGameDialog = false;
			if (change) {
				fetchMetadata();
				fetchState();
			}
		}} />
{/if}

{#if showAboutDialog}
	<AboutDialog
		{version}
		onClose={() => {
			showAboutDialog = false;
		}} />
{/if}

{#if showSettingsDialog}
	<SettingsDialog
		onClose={() => {
			showSettingsDialog = false;
			loadSettings();
		}} />
{/if}
