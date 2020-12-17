<script lang="ts">
    import { API_BASE } from "../config";

    export let onClose: (change: boolean) => void;
    export let settings: Config;

    let error = null;

    function handleClickBackrdop(event) {
        if (event.target.classList.contains("backdrop")) onClose(false);
    }

    function save() {
        fetch(API_BASE + "set_config", {
            method: "POST",
            body: JSON.stringify(settings),
        })
            .then((r) => r.json())
            .then((r) => {
                error = null;
                if (!r) error = "Empty response";
                else if (r.error !== undefined) error = r.error;
                else if (r === true) onClose(true);
                else error = "Unexpected response:\n" + JSON.stringify(r);
            });
    }
</script>

<style lang="scss">
    .settings-list {
        display: flex;
        flex-direction: column;

        > * {
            margin-top: 20px;
        }

        input[type="checkbox"] {
            height: 20px;
            width: 20px;
            vertical-align: sub;
        }
    }
</style>

<div class="backdrop" on:click={handleClickBackrdop}>
    <div class="dialog-content">
        <h1>Settings</h1>

        <div class="settings-list">
            <label for="dark-mode">
                Dark Mode:
                <select bind:value={settings.dark_mode}>
                    <option value="system">System</option>
                    <option value="dark">Dark Mode</option>
                    <option value="light">Light Mode</option>
                </select>
            </label>

            <label for="show-dev-mods">
                <input
                    id="show-dev-mods"
                    name="show-dev-mods"
                    type="checkbox"
                    bind:checked={settings.show_dev_mods} />
                Show dev mods
            </label>

            <label for="check-updates">
                <input
                    id="check-updates"
                    name="check-updates"
                    type="checkbox"
                    bind:checked={settings.check_for_updates} />
                Automatically check for updates
            </label>

            {#if error !== null}
                <p class="error">{error}</p>
            {/if}
        </div>

        <div class="footer">
            <button
                type="button"
                on:click={() => onClose(false)}>Cancel</button>
            <button
                class="submit"
                type="button"
                on:click={() => save()}>Save</button>
        </div>
    </div>
</div>
