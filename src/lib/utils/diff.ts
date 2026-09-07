interface DiffToken {
  type: 'same' | 'added' | 'removed';
  value: string;
}

export function computeWordDiff(oldText: string, newText: string): DiffToken[] {
  if (!oldText && !newText) return [];
  if (!oldText) return [{ type: 'added', value: newText }];
  if (!newText) return [{ type: 'removed', value: oldText }];
  if (oldText === newText) return [{ type: 'same', value: newText }];

  const tokenize = (s: string) => s.match(/[\wА-Яа-яЁё]+|\s+|[^\s\wА-Яа-яЁё]+/g) || [];
  const a = tokenize(oldText);
  const b = tokenize(newText);

  const result: DiffToken[] = [];
  let i = 0;
  let j = 0;
  const MAX_LOOKAHEAD = 16;

  while (i < a.length && j < b.length) {
    if (a[i] === b[j] || a[i].toLowerCase() === b[j].toLowerCase()) {
      result.push({ type: 'same', value: b[j] });
      i++;
      j++;
    } else {
      let matchA = -1;
      let matchB = -1;

      for (let offset = 1; offset <= MAX_LOOKAHEAD; offset++) {
        if (j + offset < b.length && a[i].toLowerCase() === b[j + offset].toLowerCase()) {
          matchB = j + offset;
          break;
        }
        if (i + offset < a.length && a[i + offset].toLowerCase() === b[j].toLowerCase()) {
          matchA = i + offset;
          break;
        }
      }

      if (matchB !== -1) {
        while (j < matchB) {
          result.push({ type: 'added', value: b[j] });
          j++;
        }
      } else if (matchA !== -1) {
        while (i < matchA) {
          result.push({ type: 'removed', value: a[i] });
          i++;
        }
      } else {
        result.push({ type: 'removed', value: a[i] });
        result.push({ type: 'added', value: b[j] });
        i++;
        j++;
      }
    }
  }

  while (i < a.length) {
    result.push({ type: 'removed', value: a[i] });
    i++;
  }
  while (j < b.length) {
    result.push({ type: 'added', value: b[j] });
    j++;
  }

  const merged: DiffToken[] = [];
  for (const t of result) {
    if (merged.length > 0 && merged[merged.length - 1].type === t.type) {
      merged[merged.length - 1].value += t.value;
    } else {
      merged.push({ ...t });
    }
  }
  return merged;
}
