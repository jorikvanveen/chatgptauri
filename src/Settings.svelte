<script lang="ts">
    import Input from "./lib/Input.svelte";
    import Button from "./lib/Button.svelte";

    let apiKey = "";
</script>

<h1>Settings</h1>

<div class="container">
    <Button label="Clear messages" /><br/>
    <Input label="API Key" password bind:value={apiKey} />
    <Button label="Save settings" /><br/>
</div>

<style>
    .container {
        display: flex;
        flex-direction: column;
        max-width: 50rem;
    }
</style>


