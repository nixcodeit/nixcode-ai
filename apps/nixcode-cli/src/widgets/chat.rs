    async fn send_user_message(&mut self) {
        if self.client.is_waiting().await {
            return;
        }

        let message_text = self.prompt.as_string().trim().to_string();
        if message_text.is_empty() {
            return;
        }

        // Use capability-aware message creation
        let client_clone = self.client.clone();
        let message = client_clone.create_message_with_capabilities(message_text);
        self.prompt.flush();

        self.send_message(Some(message)).await;
    }