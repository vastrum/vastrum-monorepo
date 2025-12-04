function sendmsg(msg, msg_type) {
    window.parent.postMessage(
        JSON.stringify({
            msg_type: msg_type,
            message: msg
        }),
        '*'
    );
}

function handleTransactionButtonClick(element) {
    const signature = element.getAttribute("data-signature");
    const args = element.getAttribute("data-args");
    const argsSplit = args?.split(",");

    const callobject = { signature: signature };
    argsSplit?.forEach((arg) => {
        if (arg?.includes("=")) {
            const split = arg?.split("=");
            const arg_name = split[0];
            const fixed_value = split[1];
            callobject[arg_name] = fixed_value;
        } else {
            //potential xss injection here
            //const referingID = ref.current?.querySelector(\`#\${arg} \`);

            const myInputField = document.getElementById(arg);
            callobject[arg] = myInputField?.value;
        }
    });
    const payload = JSON.stringify(callobject);
    return payload;
}
function handleRuntimeClick(event) {
    const target = event.target;
    const closestNav = target.closest("button, a");
    if (!closestNav) {
        return;
    }
    const href = closestNav.getAttribute("data-href");
    const targetAttr = closestNav.getAttribute("target");

    const isTransactionButton = closestNav.getAttribute("data-signature");
    if (href) {
        sendmsg(href, 0);
    } else if (isTransactionButton) {
        const payload = handleTransactionButtonClick(closestNav);
        sendmsg(payload, 1);
    }
}
window.addEventListener("click", handleRuntimeClick);