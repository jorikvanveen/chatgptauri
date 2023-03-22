<script lang="ts">
    import { invoke } from "@tauri-apps/api/tauri";
    import { tick } from "svelte";
    import MultilineParagraph from "./MultilineParagraph.svelte";
    import DOMpurify from "dompurify";
    import { marked } from "marked";

    class ChatMessage {
        role: "user" | "assistant" | "error"
        message: string

        constructor(role: "user" | "assistant" | "error", message: string) {
            this.role = role
            this.message = message
        }
    }

    let promptInput = "";
    let messages: ChatMessage[] = [];

    function promptKeyDown(e: KeyboardEvent) {
        if (e.key == "Enter" && !e.shiftKey) {
            e.preventDefault();
            submitPrompt(promptInput);
            promptInput = "";
        }
    }

    async function submitPrompt(prompt: string) {
        let chatlog = document.querySelector(".chatlog");

        messages.push(new ChatMessage("user", prompt))
        messages = messages
        await tick()

        chatlog.scrollTo(0, chatlog.scrollHeight)

        try {
            console.log("Requesting")
            let resp = await invoke("prompt", { prompt }) as string;
            messages.push(new ChatMessage("assistant", resp));
            console.log("Success")
        } catch (e) {
            messages.push(new ChatMessage("error", e.toString()));
            console.error(e);
        }

        messages = messages;
        await tick()
        chatlog.scrollTo(0, chatlog.scrollHeight)
    }
</script>

<main class="container">
    <div class="chatlog">
        {#each messages as message}
            {#if message.role == "user"}
                <p class="msg user"><MultilineParagraph text={message.message} /></p>
            {:else if message.role == "assistant"}
                <p class="msg assistant">{@html marked.parse(DOMpurify.sanitize(message.message))}</p>
            {:else}
                <p class="msg error"><MultilineParagraph text={message.message} /></p>
            {/if}
        {/each}
    </div>
    <div class="promptarea">
        <textarea bind:value={promptInput} on:keydown={promptKeyDown} />
    </div>
</main>

<style>
    .container {
        width: 100%;
        height: 100%;
        box-sizing: border-box; 
        padding: 2rem;

        display: flex;
        flex-direction: column;
    }

    .chatlog {
        height: 80%;
        overflow-y: scroll;
    }

    .promptarea {
        height: 20%;
    }

    .user {
        font-weight: bold;
        padding-left: 2rem;
        border-left: solid 3px var(--teal);
    }

    textarea {
        resize: none;
        outline: none;
        width: 100%;
        height: 100%;
        border-radius: 8px;
        background: var(--dark-blue);
        padding: 1rem;
        box-sizing: border-box;
        color: var(--fg);
        border: none;
    }
</style>
