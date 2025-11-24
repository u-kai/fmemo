import '@testing-library/jest-dom';
import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/react';

// Make expect globally available
globalThis.expect = expect;

// Cleanup after each test case
afterEach(() => {
  cleanup();
});