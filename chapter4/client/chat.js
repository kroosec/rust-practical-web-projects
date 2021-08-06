document.addEventListener("DOMContentLoaded", function(){
    const name = prompt("What is your name?")
    document.getElementById("name").innerText = name;
    const socket = new WebSocket("ws://localhost:8084");

    socket.onmessage = function (event) {
        const messages = document.getElementById("messages");
        const msg = JSON.parse(event.data);
        const time = (new Date(Number(msg.received_at)))
            .toLocaleString("en-US")
        messages.value += '[' + time + '] ' + msg.name + ': ' + msg.message + '\n';
    };

    const sendButton= document.getElementById("send");
    sendButton.addEventListener("click", (event) => {
        const message = document.getElementById("message");
        socket.send(
            JSON.stringify({
                name: name,
                message: message.value
            })
        )
        message.value = "";
    })
});
