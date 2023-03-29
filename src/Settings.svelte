<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import Input from "./lib/Input.svelte";
    import Button from "./lib/Button.svelte";
    import type { ChatMessage } from "./chat";
    import type { Writable } from "svelte/store";
    import { getContext } from "svelte";

    let apiKey = "";

    let messages: Writable<ChatMessage[]> = getContext("messages");

    async function clearMessages() {
        $messages = []
        await invoke("clear_messages");
        console.log("cleared")
    }
</script>

<h1>Settings</h1>

<div class="container">
    <Button on:click={clearMessages} label="Clear messages" /><br/>
    <Input label="API Key" password bind:value={apiKey} />
    <br/>
    <Button label="Save settings" /><br/>
</div>

<style>
    .container {
        display: flex;
        flex-direction: column;
        max-width: 50rem;
    }
</style>


