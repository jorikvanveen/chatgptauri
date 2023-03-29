export class ChatMessage {
    role: "user" | "assistant" | "error"
    message: string

    constructor(role: "user" | "assistant" | "error", message: string) {
        this.role = role
        this.message = message
    }
}
