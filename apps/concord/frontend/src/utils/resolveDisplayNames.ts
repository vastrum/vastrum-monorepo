import { get_user_profile, type JSDmSummary } from '../../wasm/pkg';

export async function resolveDisplayNames(dms: JSDmSummary[]): Promise<Record<string, string>> {
    const uniqueUsers = [...new Set(dms.map(dm => dm.other_user))];
    const profiles = await Promise.all(uniqueUsers.map(u => get_user_profile(u)));
    const names: Record<string, string> = {};
    for (let i = 0; i < uniqueUsers.length; i++) {
        if (profiles[i].display_name) names[uniqueUsers[i]] = profiles[i].display_name;
    }
    return names;
}
