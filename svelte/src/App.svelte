<script lang="ts">
	import AddGameDialog from "./AddGameDialog.svelte";
	import { API_BASE } from "./config";
	import ManageGameDialog from "./ManageGameDialog.svelte";

	let version: string = null;
	let bep_in_ex = null;
	let games: Map<string, SetupGame> = null;
	let meta: Metadata = null;
	let selectedGameId = null;
	let selectedGame = null;
	let showAddGameDialog = false;
	let showManageGameDialog = false;

	fetchState();
	fetchMetadata();

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
				version = state.version;
				games = new Map(Object.entries(state.games));
				selectedGameId = state.selected_game;
				selectedGame =
					selectedGameId == null ? null : games.get(selectedGameId);
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
				meta = r;
				// TODO: meta.update
			});
	}

	function handleOpenDirectory(directory) {
		// TODO
	}
</script>

<style lang="scss">
	.page-container {
		height: 100%;
		display: grid;
		grid-template-columns: 300px auto;
		grid-template-rows: 35px auto;
		grid-template-rows: 35px 70px auto;
		grid-template-areas:
			"menubar menubar"
			"game-selection game-selection"
			"sidebar main";

		&.no-game {
			grid-template-columns: auto;
			grid-template-areas:
				"menubar"
				"game-selection"
				"main";

			.main {
				display: flex;
				flex-direction: column;
				justify-content: center;
				align-items: center;
				padding: 1em;
			}
		}

		.menubar {
			grid-area: menubar;
			background-color: #f0f0f0;
			display: flex;
			justify-items: center;
			padding-left: 5px;
			border-bottom: 1px solid #ccc;

			button {
				border: none;
				padding: 0.4em 0.7em;
				background-color: transparent;

				&:hover {
					background-color: #ccc;
				}
			}
		}
		.game-selection {
			grid-area: game-selection;
			padding: 1em;

			&--label {
				font-size: 1.1em;
			}

			&--select {
				margin: 0 0.5em;
				width: calc(100% - 300px);
			}
		}
		.sidebar {
			grid-area: sidebar;
			padding: 1.3em;

			h4 {
				margin: 0;
			}

			p {
				margin: 0.2em 0;
			}

			.big-buttons {
				display: flex;
				flex-direction: column;
				margin-top: 15px;
				margin-bottom: 20px;

				button {
					margin-top: 5px;
				}
			}

			.open-directory-buttons {
				display: flex;
				flex-direction: column;
				align-items: center;

				p {
					margin-bottom: 5px;
				}

				div {
					width: 100%;
					display: flex;
					flex-direction: row;
					justify-content: space-between;

					button {
						width: 80px;
					}
				}
			}
		}
		.main {
			grid-area: main;
			padding: 1.3em;

			h4 {
				margin: 0;
			}

			.not-installed {
				display: flex;
				margin-top: 50px;
				flex-direction: column;
				align-items: center;

				h2,
				h3 {
					margin: 0.5em;
				}
			}

			&--no-game {
				margin-bottom: 30%;
			}
		}
	}
</style>

<div class="page-container" class:no-game={games?.size == 0}>
	<div class="menubar">
		<button type="button">Settings</button>
		<button type="button">About</button>
	</div>

	<div class="game-selection">
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
					No game setup. Use the button on the right to add one --&gt;
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

	{#if games?.size > 0 && selectedGame !== null}
		<div class="sidebar">
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
				<p>BepInEx v{selectedGame.bep_in_ex.version ?? '???'}</p>
			{/if}

			<div class="big-buttons">
				<button
					type="button"
					title={'Install the BepInEx Mod Loader for ' + selectedGame.game.name}>
					{selectedGame.bep_in_ex == null ? 'Install' : 'Uninstall'}
				</button>
				<button
					type="button"
					disabled={selectedGame.bep_in_ex == null}
					title={selectedGame.bep_in_ex?.enabled ? 'Enable the BepInEx Mod Loader for' : 'Disable The BepInEx Mod Loader without touching any mods or configuration. This will make the game launch completely cleanly.'}>
					{selectedGame.bep_in_ex?.enabled ? 'Enable' : 'Disable'}
				</button>
			</div>

			<!-- <div class="open-directory-buttons">
				<p>Open Directory</p>
				<div>
					<button type="button">Game</button>
					<button type="button">Mods</button>
					<button type="button">Configs</button>
				</div>
			</div> -->
		</div>
	{/if}

	<div class="main">
		{#if selectedGame}
			{#if selectedGame.bep_in_ex}
				<h4>Mods</h4>
			{:else}
				<div class="not-installed">
					<h2>The BepInEx Mod Loader is not installed</h2>
					<h3>Use the button on the left to install it</h3>
				</div>
			{/if}
		{:else}
			<h2 class="main--no-game">
				No game setup. Use the "Add Game" button above to add one.
			</h2>
		{/if}
	</div>
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
			if (change) fetchState();
		}} />
{/if}
