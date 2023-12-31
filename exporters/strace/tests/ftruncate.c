/*
 * Copyright (c) 2015-2018 Dmitry V. Levin <ldv@strace.io>
 * Copyright (c) 2015-2023 The strace developers.
 * All rights reserved.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "tests.h"
#include "scno.h"

#ifdef __NR_ftruncate

# include <stdio.h>
# include <unistd.h>

int
main(void)
{
	const kernel_ulong_t len = (kernel_ulong_t) 0xdefaced0badc0deULL;
	long rc;

	if (sizeof(len) > sizeof(long))
		rc = ftruncate(-1, len);
	else
		rc = syscall(__NR_ftruncate, -1L, len);

	printf("ftruncate(-1, %llu) = %s\n",
	       (unsigned long long) len, sprintrc(rc));

	puts("+++ exited with 0 +++");
	return 0;
}

#else

SKIP_MAIN_UNDEFINED("__NR_ftruncate")

#endif
