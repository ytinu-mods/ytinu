<script lang="ts">
    export let onClose: () => void;

    let settings = JSON.parse(localStorage.getItem("settings"));
    let dark_mode = settings?.dark_mode;
    let show_dev_mods = settings?.show_dev_mods;

    function handleClickBackrdop(event) {
        if (event.target.classList.contains("backdrop")) onClose();
    }

    function save() {
        localStorage.setItem(
            "settings",
            JSON.stringify({
                dark_mode,
                show_dev_mods,
            })
        );
        onClose();
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
                <select bind:value={dark_mode}>
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
                    bind:checked={show_dev_mods} />
                Show dev mods
            </label>
        </div>

        <div class="footer">
            <button type="button" on:click={() => onClose()}>Cancel</button>
            <button
                class="submit"
                type="button"
                on:click={() => save()}>Save</button>
        </div>
    </div>
</div>
