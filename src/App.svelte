<script lang="ts">
    import Main from "./Main.svelte"
    import SettingsSvg from "./assets/settings.svg?raw"
    import Settings from "./Settings.svelte"
    import { writable } from "svelte/store";
    import { setContext } from "svelte";
    import Page from "./page"

    let page = writable(Page.Main);
    setContext("page", page);

    function toSettings() {
        $page = Page.Settings; 
    }

    function toMain() {
        $page = Page.Main;
    }
</script>

<div class="container">
    {#if $page == Page.Main}
        <button on:click={toSettings} class="nav-button">
            {@html SettingsSvg}
        </button>
        <Main />
    {:else if $page == Page.Settings}
        <button on:click={toMain} class="nav-button">

        </button>
        <Settings />
    {/if}
</div>

<style>
    .container {
        width: 100%;
        height: 100%;
        padding: 2rem;
        box-sizing: border-box;
    }

    .nav-button {
        background: none;
        border: none;
        outline: none;
        position: fixed;
        right: 0;
        cursor: pointer;
        padding: .5rem;
        fill: var(--fg);
    }
</style>
