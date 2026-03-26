import spriteJson from './sprites/sprite.json';
import sprite2xJson from './sprites/sprite@2x.json';
import spritePngUrl from './sprites/sprite.png';
import sprite2xPngUrl from './sprites/sprite@2x.png';

// Cache decoded sprite images
let spriteImageCache: Record<string, ArrayBuffer> = {};

function dataUrlToArrayBuffer(dataUrl: string): ArrayBuffer {
    const base64 = dataUrl.split(',')[1];
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
}

/**
 * Returns sprite data for the given filename.
 * MapLibre requests:  sprite.json, sprite.png, sprite@2x.json, sprite@2x.png
 */
export async function getSpriteData(filename: string): Promise<object | ArrayBuffer> {
    if (filename.endsWith('.json')) {
        return filename.includes('@2x') ? sprite2xJson : spriteJson;
    }

    if (filename.endsWith('.png')) {
        if (!spriteImageCache[filename]) {
            const url = filename.includes('@2x') ? sprite2xPngUrl : spritePngUrl;
            spriteImageCache[filename] = dataUrlToArrayBuffer(url);
        }
        return spriteImageCache[filename];
    }

    return new ArrayBuffer(0);
}
