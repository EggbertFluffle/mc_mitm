
export async function fetchServerInfo() {
    const res = await fetch('/api/server-info');
    if (!res.ok) throw new Error(`server-info failed: ${res.status}`);
    return res.json();
}

function initServerInfo() {
    fetchServerInfo()
        .then(info => {
            document.getElementById('client-ip').textContent = info.client_ip;
            document.getElementById('server-ip').textContent = info.server_ip;
            document.getElementById('seed').textContent = info.seed;
        })
        .catch(err => console.error('Failed to load server info:', err));
}

initServerInfo();
