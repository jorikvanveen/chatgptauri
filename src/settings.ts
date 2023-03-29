type Model = "gpt3" | "gpt4";
export class Settings {
    openai_key: string;
    model: Model;

    constructor(openai_key: string, model: Model) {
        this.openai_key = openai_key;
        this.model = model;
    }
}

