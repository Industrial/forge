/**
 * BDD tests for useFilteredList hook
 * Tests verify filter state management and filtered list derivation behavior
 */
import { describe, test, expect, beforeAll } from 'bun:test'
import { renderHook, act } from '@testing-library/react'
import { useFilteredList } from './useFilteredList'
import { Window } from 'happy-dom'

// Set up DOM environment for tests
beforeAll(() => {
  const window = new Window()
  const document = window.document
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.window = window
  // @ts-expect-error - Setting global DOM APIs for bun test environment
  globalThis.document = document
})

// Test data types
type TestItem = {
  id: string
  name: string
  category: string
}

type TestFilters = {
  filterName: string
  filterCategory: string
}

// Factory function for creating test items
function createTestItem(overrides?: Partial<TestItem>): TestItem {
  return {
    id: '1',
    name: 'Test Item',
    category: 'test',
    ...overrides,
  }
}

// Simple predicate for testing
function testPredicate(item: TestItem, filters: TestFilters): boolean {
  const nameMatch =
    filters.filterName === '' ||
    item.name.toLowerCase().includes(filters.filterName.toLowerCase().trim())
  const categoryMatch =
    filters.filterCategory === '' ||
    item.category
      .toLowerCase()
      .includes(filters.filterCategory.toLowerCase().trim())
  return nameMatch && categoryMatch
}

describe('useFilteredList', () => {
  describe('export behavior', () => {
    test('should export useFilteredList as a function', () => {
      // Given the module
      // When I check the export
      // Then it should be a function
      expect(typeof useFilteredList).toBe('function')
    })
  })

  describe('initialization behavior', () => {
    test('should initialize with provided initialFilters', () => {
      // Given: items and initial filters
      const items: TestItem[] = [createTestItem()]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      // When: hook is initialized
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // Then: filters should match initialFilters
      expect(result.current[0]).toEqual(initialFilters)
    })

    test('should initialize with non-empty initialFilters', () => {
      // Given: items and initial filters with values
      const items: TestItem[] = [createTestItem()]
      const initialFilters: TestFilters = {
        filterName: 'test',
        filterCategory: 'cat',
      }

      // When: hook is initialized
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // Then: filters should match initialFilters
      expect(result.current[0]).toEqual(initialFilters)
    })

    test('should return all items when filters are empty', () => {
      // Given: items and empty filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Item 1' }),
        createTestItem({ id: '2', name: 'Item 2' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      // When: hook is initialized
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // Then: all items should be returned
      expect(result.current[2]).toHaveLength(2)
      expect(result.current[2]).toEqual(items)
    })
  })

  describe('filtering behavior', () => {
    test('should filter items by name', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
        createTestItem({ id: '3', name: 'Cherry' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: filterName is set
      act(() => {
        result.current[1]({ ...result.current[0], filterName: 'app' })
      })

      // Then: only matching items should be returned
      expect(result.current[2]).toHaveLength(1)
      expect(result.current[2][0].name).toBe('Apple')
    })

    test('should filter items by category', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Item 1', category: 'fruit' }),
        createTestItem({ id: '2', name: 'Item 2', category: 'vegetable' }),
        createTestItem({ id: '3', name: 'Item 3', category: 'fruit' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: filterCategory is set
      act(() => {
        result.current[1]({ ...result.current[0], filterCategory: 'fruit' })
      })

      // Then: only matching items should be returned
      expect(result.current[2]).toHaveLength(2)
      expect(result.current[2].every((item) => item.category === 'fruit')).toBe(
        true,
      )
    })

    test('should filter items by multiple filters (AND logic)', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple', category: 'fruit' }),
        createTestItem({ id: '2', name: 'Applesauce', category: 'fruit' }),
        createTestItem({ id: '3', name: 'Apple', category: 'vegetable' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: both filters are set
      act(() => {
        result.current[1]({
          filterName: 'app',
          filterCategory: 'fruit',
        })
      })

      // Then: only items matching both filters should be returned
      expect(result.current[2]).toHaveLength(2)
      expect(
        result.current[2].every(
          (item) =>
            item.name.toLowerCase().includes('app') &&
            item.category === 'fruit',
        ),
      ).toBe(true)
    })

    test('should be case-insensitive when predicate supports it', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'BANANA' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: filterName is set with different case
      act(() => {
        result.current[1]({ ...result.current[0], filterName: 'APPLE' })
      })

      // Then: should match regardless of case
      expect(result.current[2]).toHaveLength(1)
      expect(result.current[2][0].name).toBe('Apple')
    })

    test('should return empty array when no items match', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: filterName doesn't match any item
      act(() => {
        result.current[1]({ ...result.current[0], filterName: 'NonExistent' })
      })

      // Then: empty array should be returned
      expect(result.current[2]).toHaveLength(0)
      expect(result.current[2]).toEqual([])
    })
  })

  describe('state update behavior', () => {
    test('should update filters when setFilters is called', () => {
      // Given: hook initialized
      const items: TestItem[] = [createTestItem()]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: setFilters is called with new filters
      act(() => {
        result.current[1]({ filterName: 'new', filterCategory: 'cat' })
      })

      // Then: filters should be updated
      expect(result.current[0]).toEqual({
        filterName: 'new',
        filterCategory: 'cat',
      })
    })

    test('should update filters when setFilters is called with function updater', () => {
      // Given: hook initialized
      const items: TestItem[] = [createTestItem()]
      const initialFilters: TestFilters = {
        filterName: 'initial',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: setFilters is called with function updater
      act(() => {
        result.current[1]((prev) => ({
          ...prev,
          filterName: `${prev.filterName}-updated`,
        }))
      })

      // Then: filters should be updated
      expect(result.current[0].filterName).toBe('initial-updated')
    })

    test('should update filteredItems when filters change', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      expect(result.current[2]).toHaveLength(2)

      // When: filters are updated
      act(() => {
        result.current[1]({ ...result.current[0], filterName: 'app' })
      })

      // Then: filteredItems should be updated
      expect(result.current[2]).toHaveLength(1)
    })
  })

  describe('return value structure', () => {
    test('should return tuple with filters, setFilters, and filteredItems', () => {
      // Given: hook is used
      const items: TestItem[] = [createTestItem()]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // Then: should return tuple [filters, setFilters, filteredItems]
      expect(result.current).toBeDefined()
      expect(Array.isArray(result.current)).toBe(true)
      expect(result.current.length).toBe(3)
      expect(result.current[0]).toBeDefined()
      expect(typeof result.current[1]).toBe('function')
      expect(Array.isArray(result.current[2])).toBe(true)
    })

    test('should return setFilters as a function', () => {
      // Given: hook is used
      const items: TestItem[] = [createTestItem()]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // Then: setFilters should be a function
      expect(typeof result.current[1]).toBe('function')
    })
  })

  describe('memoization behavior', () => {
    test('should recompute filteredItems when items change', () => {
      // Given: initial items
      const initialItems: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result, rerender } = renderHook(
        ({ items }) => useFilteredList(items, initialFilters, testPredicate),
        {
          initialProps: { items: initialItems },
        },
      )

      expect(result.current[2]).toHaveLength(1)

      // When: items are updated
      const newItems: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
      ]
      rerender({ items: newItems })

      // Then: filteredItems should be recomputed
      expect(result.current[2]).toHaveLength(2)
    })

    test('should recompute filteredItems when filters change', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      expect(result.current[2]).toHaveLength(2)

      // When: filters are updated
      act(() => {
        result.current[1]({ ...result.current[0], filterName: 'app' })
      })

      // Then: filteredItems should be recomputed
      expect(result.current[2]).toHaveLength(1)
    })
  })

  describe('edge cases', () => {
    test('should handle empty items array', () => {
      // Given: empty items array
      const items: TestItem[] = []
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      // When: hook is initialized
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // Then: should return empty filteredItems
      expect(result.current[2]).toHaveLength(0)
      expect(result.current[2]).toEqual([])
    })

    test('should handle predicate that always returns false', () => {
      // Given: items and a predicate that always returns false
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }
      const alwaysFalsePredicate = () => false

      // When: hook is initialized
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, alwaysFalsePredicate),
      )

      // Then: should return empty filteredItems
      expect(result.current[2]).toHaveLength(0)
    })

    test('should handle predicate that always returns true', () => {
      // Given: items and a predicate that always returns true
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }
      const alwaysTruePredicate = () => true

      // When: hook is initialized
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, alwaysTruePredicate),
      )

      // Then: should return all items
      expect(result.current[2]).toHaveLength(2)
    })

    test('should handle rapid filter changes', () => {
      // Given: items and filters
      const items: TestItem[] = [
        createTestItem({ id: '1', name: 'Apple' }),
        createTestItem({ id: '2', name: 'Banana' }),
        createTestItem({ id: '3', name: 'Cherry' }),
      ]
      const initialFilters: TestFilters = {
        filterName: '',
        filterCategory: '',
      }

      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, testPredicate),
      )

      // When: rapid filter changes occur
      act(() => {
        result.current[1]({ ...result.current[0], filterName: 'app' })
        result.current[1]({ ...result.current[0], filterName: 'ban' })
        result.current[1]({ ...result.current[0], filterName: 'cher' })
      })

      // Then: should have final filtered state
      expect(result.current[2]).toHaveLength(1)
      expect(result.current[2][0].name).toBe('Cherry')
    })
  })

  describe('generic type behavior', () => {
    test('should work with different item types', () => {
      // Given: items of a different type
      type NumberItem = { value: number }
      const items: NumberItem[] = [{ value: 1 }, { value: 2 }, { value: 3 }]
      const initialFilters = { minValue: 0 }
      const predicate = (item: NumberItem, filters: { minValue: number }) =>
        item.value >= filters.minValue

      // When: hook is used with different types
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, predicate),
      )

      // Then: should work correctly
      expect(result.current[2]).toHaveLength(3)
    })

    test('should work with different filter types', () => {
      // Given: items and filters with different structure
      const items: TestItem[] = [createTestItem()]
      const initialFilters = {
        search: '',
        tags: [] as string[],
      }
      const predicate = (
        item: TestItem,
        filters: { search: string; tags: string[] },
      ) => {
        const searchMatch =
          filters.search === '' ||
          item.name.toLowerCase().includes(filters.search.toLowerCase())
        const tagsMatch =
          filters.tags.length === 0 || filters.tags.includes(item.category)
        return searchMatch && tagsMatch
      }

      // When: hook is used with different filter structure
      const { result } = renderHook(() =>
        useFilteredList(items, initialFilters, predicate),
      )

      // Then: should work correctly
      expect(result.current[0]).toEqual(initialFilters)
      expect(result.current[2]).toHaveLength(1)
    })
  })
})
