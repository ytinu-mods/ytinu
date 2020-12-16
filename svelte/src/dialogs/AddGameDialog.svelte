<script lang="ts">
    import { API_BASE } from "../config";

    export let onClose: (change: boolean) => void;

    let loading = true;
    let install_path = "Loading ...";
    let error = null;

    fetch(API_BASE + "find_game_directory")
        .then((r) => r.json())
        .then((r) => {
            loading = false;
            if (r !== null && r.error === undefined) install_path = r;
            else install_path = "";
        });

    function handleClickBackrdop(event) {
        if (event.target.classList.contains("backdrop")) onClose(false);
    }

    function handleClickBrowse() {
        error = null;
        try {
            fetch(API_BASE + "browse_directory")
                .then((r) => r.json())
                .then((r) => {
                    if (r !== null) {
                        if (r.error !== undefined) error = r.error;
                        else install_path = r;
                    }
                });
        } catch (e) {
            error = "Error: " + e;
        }
    }

    function handleClickSubmit() {
        try {
            fetch(API_BASE + "api/add_game", {
                method: "POST",
                body: JSON.stringify(install_path),
            })
                .then((r) => r.json())
                .then((r) => {
                    if (r !== null) {
                        if (r.error !== undefined) error = r.error;
                        else if (r === true) onClose(true);
                        else
                            error =
                                "Unexpected response:\n" + JSON.stringify(r);
                    }
                });
        } catch (e) {
            error = "Error: " + e;
        }
    }
</script>

<style lang="scss">
    .install-dir-selection {
        display: grid;
        grid-template-columns: auto 1fr 100px;
        column-gap: 5px;
        align-items: center;
    }
</style>

<div class="backdrop" on:click={handleClickBackrdop}>
    <div class="dialog-content">
        <h1>Add Game</h1>

        <div class="install-dir-selection">
            <label for="installation_dir">Installation path:</label>
            <input
                name="installation_dir"
                type="text"
                disabled={loading}
                bind:value={install_path} />
            <button type="button" on:click={handleClickBrowse}>Browse</button>
        </div>

        {#if error !== null}
            <p class="error">{error}</p>
        {/if}

        <div class="footer">
            <button
                type="button"
                on:click={() => onClose(false)}>Cancel</button>
            <button
                class="submit"
                type="button"
                on:click={handleClickSubmit}>Save</button>
        </div>
    </div>
</div>
