initSidebarItems({"constant":[["ALIGNMENT","Cache and allocation multiple alignment size"]],"fn":[["allocate_aligned","Allocates a cache-aligned memory region of `size` bytes with uninitialized values. This is more performant than using [allocate_aligned_zeroed] when all bytes will have an unknown or non-zero value and is semantically similar to `malloc`."],["allocate_aligned_zeroed","Allocates a cache-aligned memory region of `size` bytes with `0u8` on all of them. This is more performant than using [allocate_aligned] and setting all bytes to zero and is semantically similar to `calloc`."],["free_aligned","SafetyThis function is unsafe because undefined behavior can result if the caller does not ensure all of the following:"],["is_ptr_aligned",""],["memcpy","SafetyBehavior is undefined if any of the following conditions are violated:"],["reallocate","SafetyThis function is unsafe because undefined behavior can result if the caller does not ensure all of the following:"]],"static":[["ALLOCATIONS",""]]});