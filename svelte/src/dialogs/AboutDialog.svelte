<script lang="ts">
    export let onClose: () => void;
    export let version: string;

    function handleClickBackrdop(event) {
        if (event.target.classList.contains("backdrop")) onClose();
    }
</script>

<div class="backdrop" on:click={handleClickBackrdop}>
    <div class="dialog-content">
        <h1>About</h1>

        <p>
            ytinu Mod Manager
            {#if version}v{version}{:else}- Unknown Version{/if}
        </p>

        <p>using BepInEx v5.4 as Mod Loader</p>

        <p>Source Code: https://github.com/ytinu-mods/ytinu</p>

        <p>BepInEx: https://github.com/BepInEx/BepInEx</p>

        <p>
            For help, suggestions, and bug reports open an issue at
            https://github.com/ytinu-mods/ytinu/issues, ask on the official
            Desperados or Mimimi Discord in the Desperados 3 modding channels or
            PM @1vader on Discord via one of those servers.
        </p>
    </div>
</div>
