document.addEventListener('DOMContentLoaded', () => {
    hljs.initHighlightingOnLoad();
    const requestForm = document.getElementById('request-form');
    const queryApiBtn = document.getElementById('query-api-btn');
    const apiSelect = document.getElementById('api-select');
    const filterInput = document.getElementById('filter-input');
    const queryUrlInput = document.getElementById('query-url-input');

    queryApiBtn.addEventListener('click', () => {
        const apiQueryUrl = queryUrlInput.value;
        if (!apiQueryUrl) {
            console.error('请输入查询 API 的 URL');
            return;
        }
        fetch(apiQueryUrl)
           .then(response => {
                if (!response.ok) {
                    throw new Error(`请求失败，状态码：${response.status}`);
                }
                return response.json();
            })
           .then(data => {
                // 清空现有的选项
                apiSelect.innerHTML = '';
                data.forEach(apiInfo => {
                    if (!filterInput.value || apiInfo.url.includes(filterInput.value) || apiInfo.method.includes(filterInput.value)) {
                        const option = document.createElement('option');
                        option.value = apiInfo.url;
                        option.textContent = `${apiInfo.method} - ${apiInfo.url}`;
                        apiSelect.appendChild(option);
                    }
                });
            })
           .catch(error => {
                console.error('查询 API 信息失败:', error);
            });
    });

    requestForm.addEventListener('submit', (event) => {
        event.preventDefault();
        const method = document.getElementById('method').value;
        const url = document.getElementById('url').value;
        const params = document.getElementById('params').value;
        const authToken = document.getElementById('auth-token').value;
        fetch(url, {
            method,
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `${authToken}`
            },
            body: method!== 'GET'? params : undefined
        })
           .then(response => Promise.all([response.status, response.json()]))
           .then(([responseCode, responseJson]) => {
                const formattedJson = JSON.stringify(responseJson, null, 4);
                const resultDiv = document.getElementById('result');
                resultDiv.innerHTML = `
                    <div class="response-code">响应代码: ${responseCode}</div>
                    <div class="response-body">
                        <pre><code class="json">${formattedJson}</code></pre>
                    </div>
                `;
                hljs.highlightBlock(document.querySelector('pre code'));
            })
           .catch(error => {
                console.error('错误:', error);
                const resultDiv = document.getElementById('result');
                resultDiv.textContent = `请求失败: ${error.message}`;
            });
    });

    // 监听选择框变化
    apiSelect.addEventListener('change', () => {
        const selectedUrl = apiSelect.value;
        const selectedMethod = apiSelect.options[apiSelect.selectedIndex].textContent.split(' - ')[0];
        document.getElementById('method').value = selectedMethod;
        document.getElementById('url').value = selectedUrl;
    });

    // 监听过滤输入框变化
    filterInput.addEventListener('input', () => {
        queryApiBtn.click();
    });
});