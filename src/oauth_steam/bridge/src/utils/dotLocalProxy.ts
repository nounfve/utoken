import { type ProxyOptions } from 'vite'

const PORT_REGEX = /^\/.local\/(\d+)\//;

export const dotLocal = (): Record<string, string | ProxyOptions> => ({
    '/.local': {
        target: 'http://localhost:8000', // Initial placeholder target
        changeOrigin: true,
        secure: false,
        configure: (proxy, options) => {
            proxy.on('proxyReq', (proxyReq, req) => {
                const targetPort = req.url?.match(PORT_REGEX)?.at(1)
                if (targetPort) {
                    options.target = `http://localhost:${targetPort}`;
                    proxyReq.path = req.url!.replace(PORT_REGEX, '/');
                    console.log(`[Proxying] ${options.target}${proxyReq.path}`);
                }
            });
        },
    },
})