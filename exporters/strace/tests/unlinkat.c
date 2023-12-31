/*
 * Check decoding of unlinkat syscall.
 *
 * Copyright (c) 2016-2023 The strace developers.
 * All rights reserved.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "tests.h"
#include "scno.h"

#include <stdio.h>
#include <unistd.h>

int
main(void)
{
	static const char sample[] = "unlinkat_sample";
	const long fd = (long) 0xdeadbeefffffffffULL;

	long rc = syscall(__NR_unlinkat, fd, sample, 0);
	printf("unlinkat(%d, \"%s\", 0) = %s\n",
	       (int) fd, sample, sprintrc(rc));

	rc = syscall(__NR_unlinkat, -100, sample, -1L);
	printf("unlinkat(%s, \"%s\", %s) = %s\n",
	       "AT_FDCWD", sample,
	       "AT_SYMLINK_NOFOLLOW|AT_REMOVEDIR|AT_SYMLINK_FOLLOW"
	       "|AT_NO_AUTOMOUNT|AT_EMPTY_PATH|AT_RECURSIVE|0xffff60ff",
	       sprintrc(rc));

	puts("+++ exited with 0 +++");
	return 0;
}
