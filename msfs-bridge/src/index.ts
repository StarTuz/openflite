/**
 * OpenFlite MSFS Bridge
 * 
 * This WASM gauge runs inside MSFS and exposes SimVars via HTTP.
 * Port: 8080 (configurable)
 */

// SimVar definitions to subscribe to by default
const DEFAULT_SIMVARS: Array<{ name: string; unit: string }> = [
    { name: 'INDICATED ALTITUDE', unit: 'feet' },
    { name: 'AIRSPEED INDICATED', unit: 'knots' },
    { name: 'HEADING INDICATOR', unit: 'degrees' },
    { name: 'GEAR HANDLE POSITION', unit: 'percent' },
    { name: 'AUTOPILOT MASTER', unit: 'bool' },
    { name: 'FLAPS HANDLE PERCENT', unit: 'percent' },
    { name: 'NAV1 ACTIVE FREQUENCY', unit: 'mhz' },
    { name: 'COM1 ACTIVE FREQUENCY', unit: 'mhz' },
    { name: 'TRANSPONDER CODE', unit: 'number' },
];

interface SimVarCache {
    [name: string]: number;
}

const simvarCache: SimVarCache = {};

/**
 * Initialize the bridge - called when gauge loads
 */
export function init(): void {
    console.log('[OpenFlite Bridge] Initializing...');

    // Subscribe to default SimVars
    for (const sv of DEFAULT_SIMVARS) {
        // In real implementation: SimVar.Register(sv.name, sv.unit)
        simvarCache[sv.name] = 0;
    }

    // Start HTTP server
    startHttpServer(8080);
}

/**
 * Update loop - called every frame
 */
export function update(): void {
    // Poll SimVars and update cache
    for (const sv of DEFAULT_SIMVARS) {
        // In real implementation: simvarCache[sv.name] = SimVar.GetValue(sv.name, sv.unit);
    }
}

/**
 * Start the HTTP server
 * Note: MSFS WASM has limited networking, may need alternative approach
 */
function startHttpServer(port: number): void {
    console.log(`[OpenFlite Bridge] HTTP server would start on port ${port}`);

    // In real implementation, use MSFS coherent networking or
    // write values to a shared file that a host process reads
}

/**
 * Handle GET /simvars request
 */
export function handleGetSimvars(): SimVarCache {
    return { ...simvarCache };
}

/**
 * Handle POST /command request
 */
export function handleCommand(event: string): boolean {
    console.log(`[OpenFlite Bridge] Executing: ${event}`);
    // In real implementation: SimVar.SetValue('K:' + event, 'number', 1);
    return true;
}

/**
 * Handle POST /simvar request
 */
export function handleSetSimvar(name: string, value: number): boolean {
    console.log(`[OpenFlite Bridge] Setting ${name} = ${value}`);
    // In real implementation: SimVar.SetValue(name, 'number', value);
    simvarCache[name] = value;
    return true;
}
