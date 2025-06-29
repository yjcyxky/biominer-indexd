type MergeHow = 'inner' | 'left' | 'right' | 'outer';

interface MergeOptions {
    leftOn: string;
    rightOn: string;
    how?: MergeHow;
}

export const mergeData = (
    data1: Record<string, any>[],
    data2: Record<string, any>[],
    options: MergeOptions
): Record<string, any>[] => {
    const { leftOn, rightOn, how = 'inner' } = options;

    const data2Index = new Map<any, Record<string, any>[]>();
    data2.forEach(r => {
        const key = r[rightOn];
        if (!data2Index.has(key)) {
            data2Index.set(key, []);
        }
        data2Index.get(key)!.push(r);
    });

    const result: Record<string, any>[] = [];

    const matchedKeys = new Set<any>();

    data1.forEach(leftRow => {
        const key = leftRow[leftOn];
        const matchedRows = data2Index.get(key);
        if (matchedRows && matchedRows.length > 0) {
            matchedKeys.add(key);
            matchedRows.forEach(rightRow => {
                result.push({ ...leftRow, ...rightRow });
            });
        } else if (how === 'left' || how === 'outer') {
            result.push({ ...leftRow });
        }
    });

    if (how === 'right' || how === 'outer') {
        data2.forEach(rightRow => {
            const key = rightRow[rightOn];
            if (!matchedKeys.has(key)) {
                result.push({ ...rightRow });
            }
        });
    }

    return result;
};
