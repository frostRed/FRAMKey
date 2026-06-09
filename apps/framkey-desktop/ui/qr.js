(() => {
  "use strict";

  const QR_BYTE_VERSIONS = [
    { version: 1, size: 21, dataCodewords: 19, eccCodewords: 7, align: [] },
    { version: 2, size: 25, dataCodewords: 34, eccCodewords: 10, align: [6, 18] },
    { version: 3, size: 29, dataCodewords: 55, eccCodewords: 15, align: [6, 22] },
    { version: 4, size: 33, dataCodewords: 80, eccCodewords: 20, align: [6, 26] },
    { version: 5, size: 37, dataCodewords: 108, eccCodewords: 26, align: [6, 30] },
  ];

  function svgForText(text) {
    const matrix = matrixForText(text);
    const quietZone = 4;
    const viewSize = matrix.length + quietZone * 2;
    const path = [];
    for (let y = 0; y < matrix.length; y += 1) {
      for (let x = 0; x < matrix.length; x += 1) {
        if (matrix[y][x]) {
          path.push(`M${x + quietZone},${y + quietZone}h1v1h-1z`);
        }
      }
    }
    return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${viewSize} ${viewSize}" role="img" aria-label="Receive address QR code" shape-rendering="crispEdges"><rect width="${viewSize}" height="${viewSize}" fill="#fff"/><path fill="#111827" d="${path.join("")}"/></svg>`;
  }

  function matrixForText(text) {
    const bytes = Array.from(new TextEncoder().encode(text));
    const requiredBits = 4 + 8 + bytes.length * 8;
    const spec = QR_BYTE_VERSIONS.find((candidate) => requiredBits <= candidate.dataCodewords * 8);
    if (!spec) {
      throw new Error("Address is too long for QR");
    }

    const data = dataCodewords(bytes, spec.dataCodewords);
    const ecc = reedSolomonRemainder(data, reedSolomonDivisor(spec.eccCodewords));
    const codewords = data.concat(ecc);
    let bestMatrix = null;
    let bestPenalty = Infinity;
    for (let mask = 0; mask < 8; mask += 1) {
      const matrix = buildMatrix(spec, codewords, mask);
      const penalty = penaltyScore(matrix);
      if (penalty < bestPenalty) {
        bestMatrix = matrix;
        bestPenalty = penalty;
      }
    }
    return bestMatrix;
  }

  function dataCodewords(bytes, capacityCodewords) {
    const bits = [];
    appendBits(bits, 0x4, 4);
    appendBits(bits, bytes.length, 8);
    for (const byte of bytes) {
      appendBits(bits, byte, 8);
    }

    const capacityBits = capacityCodewords * 8;
    appendBits(bits, 0, Math.min(4, capacityBits - bits.length));
    while (bits.length % 8 !== 0) {
      bits.push(false);
    }

    const codewords = [];
    for (let offset = 0; offset < bits.length; offset += 8) {
      let value = 0;
      for (let bit = 0; bit < 8; bit += 1) {
        value = (value << 1) | (bits[offset + bit] ? 1 : 0);
      }
      codewords.push(value);
    }
    for (let pad = 0; codewords.length < capacityCodewords; pad += 1) {
      codewords.push(pad % 2 === 0 ? 0xec : 0x11);
    }
    return codewords;
  }

  function appendBits(bits, value, length) {
    for (let bit = length - 1; bit >= 0; bit -= 1) {
      bits.push(((value >>> bit) & 1) !== 0);
    }
  }

  function reedSolomonDivisor(degree) {
    const result = new Array(degree).fill(0);
    result[degree - 1] = 1;
    let root = 1;
    for (let i = 0; i < degree; i += 1) {
      for (let j = 0; j < degree; j += 1) {
        result[j] = gfMultiply(result[j], root);
        if (j + 1 < degree) {
          result[j] ^= result[j + 1];
        }
      }
      root = gfMultiply(root, 0x02);
    }
    return result;
  }

  function reedSolomonRemainder(data, divisor) {
    const result = new Array(divisor.length).fill(0);
    for (const byte of data) {
      const factor = byte ^ result.shift();
      result.push(0);
      for (let i = 0; i < divisor.length; i += 1) {
        result[i] ^= gfMultiply(divisor[i], factor);
      }
    }
    return result;
  }

  function gfMultiply(left, right) {
    let product = 0;
    let factor = left;
    let value = right;
    while (value > 0) {
      if ((value & 1) !== 0) {
        product ^= factor;
      }
      factor <<= 1;
      if ((factor & 0x100) !== 0) {
        factor ^= 0x11d;
      }
      value >>>= 1;
    }
    return product;
  }

  function buildMatrix(spec, codewords, mask) {
    const size = spec.size;
    const modules = Array.from({ length: size }, () => new Array(size).fill(false));
    const reserved = Array.from({ length: size }, () => new Array(size).fill(false));
    const setFunction = (x, y, dark) => {
      if (x < 0 || y < 0 || x >= size || y >= size) {
        return;
      }
      modules[y][x] = Boolean(dark);
      reserved[y][x] = true;
    };

    drawFunctionPatterns(spec, setFunction);
    const bits = codewordBits(codewords);
    let bitIndex = 0;
    let upward = true;
    for (let right = size - 1; right >= 1; right -= 2) {
      if (right === 6) {
        right -= 1;
      }
      for (let vertical = 0; vertical < size; vertical += 1) {
        const y = upward ? size - 1 - vertical : vertical;
        for (let column = 0; column < 2; column += 1) {
          const x = right - column;
          if (reserved[y][x]) {
            continue;
          }
          const bit = bitIndex < bits.length ? bits[bitIndex] : false;
          modules[y][x] = bit !== maskBit(mask, x, y);
          bitIndex += 1;
        }
      }
      upward = !upward;
    }
    setFormatBits(size, setFunction, mask);
    return modules;
  }

  function codewordBits(codewords) {
    const bits = [];
    for (const codeword of codewords) {
      appendBits(bits, codeword, 8);
    }
    return bits;
  }

  function drawFunctionPatterns(spec, setFunction) {
    const size = spec.size;
    drawFinderPattern(setFunction, 0, 0);
    drawFinderPattern(setFunction, size - 7, 0);
    drawFinderPattern(setFunction, 0, size - 7);
    for (let i = 8; i < size - 8; i += 1) {
      const dark = i % 2 === 0;
      setFunction(i, 6, dark);
      setFunction(6, i, dark);
    }
    for (const x of spec.align) {
      for (const y of spec.align) {
        const overlapsFinder =
          (x === 6 && y === 6) || (x === 6 && y === size - 7) || (x === size - 7 && y === 6);
        if (!overlapsFinder) {
          drawAlignmentPattern(setFunction, x, y);
        }
      }
    }
    setFormatBits(size, setFunction, 0);
    setFunction(8, size - 8, true);
  }

  function drawFinderPattern(setFunction, left, top) {
    for (let y = -1; y <= 7; y += 1) {
      for (let x = -1; x <= 7; x += 1) {
        const horizontal = left + x;
        const vertical = top + y;
        const inFinder = x >= 0 && x <= 6 && y >= 0 && y <= 6;
        const dark =
          inFinder &&
          (x === 0 || x === 6 || y === 0 || y === 6 || (x >= 2 && x <= 4 && y >= 2 && y <= 4));
        setFunction(horizontal, vertical, dark);
      }
    }
  }

  function drawAlignmentPattern(setFunction, centerX, centerY) {
    for (let y = -2; y <= 2; y += 1) {
      for (let x = -2; x <= 2; x += 1) {
        const distance = Math.max(Math.abs(x), Math.abs(y));
        setFunction(centerX + x, centerY + y, distance === 2 || distance === 0);
      }
    }
  }

  function setFormatBits(size, setFunction, mask) {
    const bits = formatBits(mask);
    for (let i = 0; i <= 5; i += 1) {
      setFunction(8, i, bit(bits, i));
    }
    setFunction(8, 7, bit(bits, 6));
    setFunction(8, 8, bit(bits, 7));
    setFunction(7, 8, bit(bits, 8));
    for (let i = 9; i < 15; i += 1) {
      setFunction(14 - i, 8, bit(bits, i));
    }
    for (let i = 0; i < 8; i += 1) {
      setFunction(size - 1 - i, 8, bit(bits, i));
    }
    for (let i = 8; i < 15; i += 1) {
      setFunction(8, size - 15 + i, bit(bits, i));
    }
  }

  function formatBits(mask) {
    const errorCorrectionLow = 1;
    const data = (errorCorrectionLow << 3) | mask;
    let remainder = data;
    for (let i = 0; i < 10; i += 1) {
      remainder = (remainder << 1) ^ (((remainder >>> 9) & 1) !== 0 ? 0x537 : 0);
    }
    return ((data << 10) | (remainder & 0x3ff)) ^ 0x5412;
  }

  function bit(value, index) {
    return ((value >>> index) & 1) !== 0;
  }

  function maskBit(mask, x, y) {
    if (mask === 0) {
      return (x + y) % 2 === 0;
    }
    if (mask === 1) {
      return y % 2 === 0;
    }
    if (mask === 2) {
      return x % 3 === 0;
    }
    if (mask === 3) {
      return (x + y) % 3 === 0;
    }
    if (mask === 4) {
      return (Math.floor(y / 2) + Math.floor(x / 3)) % 2 === 0;
    }
    if (mask === 5) {
      return ((x * y) % 2) + ((x * y) % 3) === 0;
    }
    if (mask === 6) {
      return (((x * y) % 2) + ((x * y) % 3)) % 2 === 0;
    }
    return (((x + y) % 2) + ((x * y) % 3)) % 2 === 0;
  }

  function penaltyScore(matrix) {
    let penalty = 0;
    penalty += runPenalty(matrix, true);
    penalty += runPenalty(matrix, false);
    penalty += blockPenalty(matrix);
    penalty += finderLikePenalty(matrix, true);
    penalty += finderLikePenalty(matrix, false);
    penalty += balancePenalty(matrix);
    return penalty;
  }

  function runPenalty(matrix, rows) {
    let penalty = 0;
    const size = matrix.length;
    for (let outer = 0; outer < size; outer += 1) {
      let runColor = false;
      let runLength = 0;
      for (let inner = 0; inner < size; inner += 1) {
        const color = rows ? matrix[outer][inner] : matrix[inner][outer];
        if (inner === 0 || color !== runColor) {
          if (runLength >= 5) {
            penalty += runLength - 2;
          }
          runColor = color;
          runLength = 1;
        } else {
          runLength += 1;
        }
      }
      if (runLength >= 5) {
        penalty += runLength - 2;
      }
    }
    return penalty;
  }

  function blockPenalty(matrix) {
    let penalty = 0;
    for (let y = 0; y < matrix.length - 1; y += 1) {
      for (let x = 0; x < matrix.length - 1; x += 1) {
        const color = matrix[y][x];
        if (color === matrix[y][x + 1] && color === matrix[y + 1][x] && color === matrix[y + 1][x + 1]) {
          penalty += 3;
        }
      }
    }
    return penalty;
  }

  function finderLikePenalty(matrix, rows) {
    let penalty = 0;
    const size = matrix.length;
    const pattern = [true, false, true, true, true, false, true];
    for (let outer = 0; outer < size; outer += 1) {
      for (let start = 0; start <= size - pattern.length; start += 1) {
        let matches = true;
        for (let offset = 0; offset < pattern.length; offset += 1) {
          const value = rows ? matrix[outer][start + offset] : matrix[start + offset][outer];
          if (value !== pattern[offset]) {
            matches = false;
            break;
          }
        }
        if (!matches) {
          continue;
        }
        const leadingLight = allLight(matrix, rows, outer, start - 4, start);
        const trailingLight = allLight(matrix, rows, outer, start + pattern.length, start + pattern.length + 4);
        if (leadingLight || trailingLight) {
          penalty += 40;
        }
      }
    }
    return penalty;
  }

  function allLight(matrix, rows, outer, from, to) {
    if (from < 0 || to > matrix.length) {
      return false;
    }
    for (let index = from; index < to; index += 1) {
      const value = rows ? matrix[outer][index] : matrix[index][outer];
      if (value) {
        return false;
      }
    }
    return true;
  }

  function balancePenalty(matrix) {
    let dark = 0;
    for (const row of matrix) {
      for (const value of row) {
        if (value) {
          dark += 1;
        }
      }
    }
    const total = matrix.length * matrix.length;
    return Math.floor(Math.abs(dark * 20 - total * 10) / total) * 10;
  }

  window.framkeyQrSvgForText = svgForText;
})();
