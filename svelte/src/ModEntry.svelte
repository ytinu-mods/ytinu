<script type="ts">
    export let available_mod: Mod = null;
    export let installed_mod: InstalledMod = null;
    export let onInstall: (id: string) => void = null;
    export let onUninstall: (id: string) => void = null;
    export let onUpdate: (id: string) => void = null;
    // export let onEnable: (id: string) => void = null;

    export let expandedMod: string;
    let mod = available_mod || installed_mod.m;
    let hasUpdate =
        installed_mod && installed_mod.version !== installed_mod.m.version;

    function stopPropagation(event: MouseEvent, func) {
        event.stopPropagation();
        func();
    }
</script>

<style lang="scss">
    .mod {
        display: grid;
        grid-template-columns: 40px auto 75px 120px;
        grid-template-rows: 40px;
        grid-template-areas: "checkbox-enabled name version btn-install";
        align-items: center;
        border-bottom: 1px solid var(--color-border-primary);
        cursor: pointer;

        &.expanded {
            grid-template-rows: 40px auto;
            grid-template-areas: "checkbox-enabled name version btn-install" "x desc desc desc";
        }

        &.installed-mod {
            grid-template-columns: 40px auto 75px 120px 120px;
            grid-template-areas: "checkbox-enabled name version btn-update btn-install";

            &.expanded {
                grid-template-rows: 40px auto;
                grid-template-areas: "checkbox-enabled name version btn-update btn-install" "x desc desc desc desc";
            }
        }

        // input[type="checkbox"] {
        //     grid-area: checkbox-enabled;
        //     width: 22px;
        //     height: 22px;
        //     justify-self: center;
        // }

        .mod-name {
            grid-area: name;
        }

        .mod-version {
            grid-area: version;
            justify-self: right;
            margin: 0 0.4em;
        }

        button {
            margin: 0 0.4em;

            &.btn-update {
                grid-area: btn-update;
            }
            &.btn-install {
                grid-area: btn-install;
            }
        }

        .mod-description {
            grid-area: desc;
            background-color: var(--color-menu-hover);
            padding-left: 1.2em;
            cursor: initial;
        }
    }
</style>

<div
    class="mod"
    class:installed-mod={!!installed_mod}
    class:expanded={expandedMod === mod.id}
    on:click={() => (expandedMod = expandedMod === mod.id ? null : mod.id)}>
    <!-- <input
        type="checkbox"
        disabled={!installed_mod}
        checked={installed_mod?.enabled}
        on:change={() => onEnable(mod.id)} /> -->
    <span class="mod-name">{mod.name}</span>
    <span class="mod-version" class:error={hasUpdate}>v{mod.version}</span>
    {#if onUpdate}
        <button
            type="button"
            class="btn-update"
            disabled={!hasUpdate}
            title={hasUpdate ? 'Update to v' + mod.version : 'Latest version installed'}
            on:click={(e) => stopPropagation(e, () => onUpdate(mod.id))}>
            Update
        </button>
    {/if}
    {#if onInstall}
        <button
            type="button"
            class="btn-install"
            on:click={(e) => stopPropagation(e, () => onInstall(mod.id))}>
            Install
        </button>
    {:else}
        <button
            type="button"
            class="btn-install"
            on:click={(e) => stopPropagation(e, () => onUninstall(mod.id))}>
            Uninstall
        </button>
    {/if}
    {#if expandedMod === mod.id}
        <div class="mod-description" on:click={(e) => e.stopPropagation()}>
            <p>{mod.description}</p>
            {#if mod.source && mod.source === mod.homepage}
                <p>Source/Homepage: {mod.source}</p>
            {:else}
                {#if mod.source}
                    <p>Source: {mod.source}</p>
                {/if}
                {#if mod.homepage}
                    <p>Homepage: {mod.homepage}</p>
                {/if}
            {/if}
        </div>
    {/if}
</div>
