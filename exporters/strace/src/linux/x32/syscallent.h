/*
 * Copyright (c) 2012-2023 The strace developers.
 * All rights reserved.
 *
 * SPDX-License-Identifier: LGPL-2.1-or-later
 */

[  0] = { 3,	TD,		SEN(read),			"read"			},
[  1] = { 3,	TD,		SEN(write),			"write"			},
[  2] = { 3,	TD|TF,		SEN(open),			"open"			},
[  3] = { 1,	TD,		SEN(close),			"close"			},
[  4] = { 2,	TF|TST|TSTA,	SEN(stat),			"stat"			},
[  5] = { 2,	TD|TFST|TSTA,	SEN(fstat),			"fstat"			},
[  6] = { 2,	TF|TLST|TSTA,	SEN(lstat),			"lstat"			},
[  7] = { 3,	TD,		SEN(poll_time64),		"poll"			},
[  8] = { 3,	TD,		SEN(lseek),			"lseek"			},
[  9] = { 6,	TD|TM|SI,	SEN(mmap),			"mmap"			},
[ 10] = { 3,	TM|SI,		SEN(mprotect),			"mprotect"		},
[ 11] = { 2,	TM|SI,		SEN(munmap),			"munmap"		},
[ 12] = { 1,	TM|SI,		SEN(brk),			"brk"			},
[ 13] = { 4,	TS,		SEN(printargs),			"rt_sigaction#64"	},
[ 14] = { 4,	TS,		SEN(rt_sigprocmask),		"rt_sigprocmask"	},
[ 15] = { 0,	TS,		SEN(rt_sigreturn),		"rt_sigreturn#64"	},
[ 16] = { 3,	TD,		SEN(printargs),			"ioctl#64"		},
[ 17] = { 4,	TD,		SEN(pread),			"pread64"		},
[ 18] = { 4,	TD,		SEN(pwrite),			"pwrite64"		},
[ 19] = { 3,	TD,		SEN(printargs),			"readv#64"		},
[ 20] = { 3,	TD,		SEN(printargs),			"writev#64"		},
[ 21] = { 2,	TF,		SEN(access),			"access"		},
[ 22] = { 1,	TD,		SEN(pipe),			"pipe"			},
[ 23] = { 5,	TD,		SEN(select),			"select"		},
[ 24] = { 0,	0,		SEN(sched_yield),		"sched_yield"		},
[ 25] = { 5,	TM|SI,		SEN(mremap),			"mremap"		},
[ 26] = { 3,	TM,		SEN(msync),			"msync"			},
[ 27] = { 3,	TM,		SEN(mincore),			"mincore"		},
[ 28] = { 3,	TM,		SEN(madvise),			"madvise"		},
[ 29] = { 3,	TI,		SEN(shmget),			"shmget"		},
[ 30] = { 3,	TI|TM|SI,	SEN(shmat),			"shmat"			},
[ 31] = { 3,	TI,		SEN(shmctl),			"shmctl"		},
[ 32] = { 1,	TD,		SEN(dup),			"dup"			},
[ 33] = { 2,	TD,		SEN(dup2),			"dup2"			},
[ 34] = { 0,	TS,		SEN(pause),			"pause"			},
[ 35] = { 2,	0,		SEN(nanosleep_time64),		"nanosleep"		},
[ 36] = { 2,	0,		SEN(getitimer),			"getitimer"		},
[ 37] = { 1,	0,		SEN(alarm),			"alarm"			},
[ 38] = { 3,	0,		SEN(setitimer),			"setitimer"		},
[ 39] = { 0,	PU|NF,		SEN(getpid),			"getpid"		},
[ 40] = { 4,	TD|TN,		SEN(sendfile64),		"sendfile"		},
[ 41] = { 3,	TN,		SEN(socket),			"socket"		},
[ 42] = { 3,	TN,		SEN(connect),			"connect"		},
[ 43] = { 3,	TN,		SEN(accept),			"accept"		},
[ 44] = { 6,	TN,		SEN(sendto),			"sendto"		},
[ 45] = { 6,	TN,		SEN(printargs),			"recvfrom#64"		},
[ 46] = { 3,	TN,		SEN(printargs),			"sendmsg#64"		},
[ 47] = { 3,	TN,		SEN(printargs),			"recvmsg#64"		},
[ 48] = { 2,	TN,		SEN(shutdown),			"shutdown"		},
[ 49] = { 3,	TN,		SEN(bind),			"bind"			},
[ 50] = { 2,	TN,		SEN(listen),			"listen"		},
[ 51] = { 3,	TN,		SEN(getsockname),		"getsockname"		},
[ 52] = { 3,	TN,		SEN(getpeername),		"getpeername"		},
[ 53] = { 4,	TN,		SEN(socketpair),		"socketpair"		},
[ 54] = { 5,	TN,		SEN(printargs),			"setsockopt#64"		},
[ 55] = { 5,	TN,		SEN(printargs),			"getsockopt#64"		},
[ 56] = { 5,	TP,		SEN(clone),			"clone"			},
[ 57] = { 0,	TP,		SEN(fork),			"fork"			},
[ 58] = { 0,	TP,		SEN(vfork),			"vfork"			},
[ 59] = { 3,	CC|TF|TP|TSD|SE|SI,	SEN(printargs),			"execve#64"		},
[ 60] = { 1,	TP|SE,		SEN(exit),			"exit"			},
[ 61] = { 4,	TP,		SEN(wait4),			"wait4"			},
[ 62] = { 2,	TS|TP,		SEN(kill),			"kill"			},
[ 63] = { 1,	0,		SEN(uname),			"uname"			},
[ 64] = { 3,	TI,		SEN(semget),			"semget"		},
[ 65] = { 3,	TI,		SEN(semop),			"semop"			},
[ 66] = { 4,	TI,		SEN(semctl),			"semctl"		},
[ 67] = { 1,	TI|TM|SI,	SEN(shmdt),			"shmdt"			},
[ 68] = { 2,	TI,		SEN(msgget),			"msgget"		},
[ 69] = { 4,	TI,		SEN(msgsnd),			"msgsnd"		},
[ 70] = { 5,	TI,		SEN(msgrcv),			"msgrcv"		},
[ 71] = { 3,	TI,		SEN(msgctl),			"msgctl"		},
[ 72] = { 3,	TD,		SEN(fcntl),			"fcntl"			},
[ 73] = { 2,	TD,		SEN(flock),			"flock"			},
[ 74] = { 1,	TD,		SEN(fsync),			"fsync"			},
[ 75] = { 1,	TD,		SEN(fdatasync),			"fdatasync"		},
[ 76] = { 2,	TF,		SEN(truncate64),		"truncate"		},
[ 77] = { 2,	TD,		SEN(ftruncate64),		"ftruncate"		},
[ 78] = { 3,	TD,		SEN(getdents),			"getdents"		},
[ 79] = { 2,	TF,		SEN(getcwd),			"getcwd"		},
[ 80] = { 1,	TF,		SEN(chdir),			"chdir"			},
[ 81] = { 1,	TD,		SEN(fchdir),			"fchdir"		},
[ 82] = { 2,	TF,		SEN(rename),			"rename"		},
[ 83] = { 2,	TF,		SEN(mkdir),			"mkdir"			},
[ 84] = { 1,	TF,		SEN(rmdir),			"rmdir"			},
[ 85] = { 2,	TD|TF,		SEN(creat),			"creat"			},
[ 86] = { 2,	TF,		SEN(link),			"link"			},
[ 87] = { 1,	TF,		SEN(unlink),			"unlink"		},
[ 88] = { 2,	TF,		SEN(symlink),			"symlink"		},
[ 89] = { 3,	TF,		SEN(readlink),			"readlink"		},
[ 90] = { 2,	TF,		SEN(chmod),			"chmod"			},
[ 91] = { 2,	TD,		SEN(fchmod),			"fchmod"		},
[ 92] = { 3,	TF,		SEN(chown),			"chown"			},
[ 93] = { 3,	TD,		SEN(fchown),			"fchown"		},
[ 94] = { 3,	TF,		SEN(chown),			"lchown"		},
[ 95] = { 1,	NF,		SEN(umask),			"umask"			},
[ 96] = { 2,	TCL,		SEN(gettimeofday),		"gettimeofday"		},
[ 97] = { 2,	0,		SEN(getrlimit),			"getrlimit"		},
[ 98] = { 2,	0,		SEN(getrusage),			"getrusage"		},
[ 99] = { 1,	0,		SEN(sysinfo),			"sysinfo"		},
[100] = { 1,	0,		SEN(times),			"times"			},
[101] = { 4,	0,		SEN(printargs),			"ptrace#64"		},
[102] = { 0,	TC|PU|NF,	SEN(getuid),			"getuid"		},
[103] = { 3,	0,		SEN(syslog),			"syslog"		},
[104] = { 0,	TC|PU|NF,	SEN(getgid),			"getgid"		},
[105] = { 1,	TC,		SEN(setuid),			"setuid"		},
[106] = { 1,	TC,		SEN(setgid),			"setgid"		},
[107] = { 0,	TC|PU|NF,	SEN(geteuid),			"geteuid"		},
[108] = { 0,	TC|PU|NF,	SEN(getegid),			"getegid"		},
[109] = { 2,	0,		SEN(setpgid),			"setpgid"		},
[110] = { 0,	PU|NF,		SEN(getppid),			"getppid"		},
[111] = { 0,	PU|NF,		SEN(getpgrp),			"getpgrp"		},
[112] = { 0,	0,		SEN(setsid),			"setsid"		},
[113] = { 2,	TC,		SEN(setreuid),			"setreuid"		},
[114] = { 2,	TC,		SEN(setregid),			"setregid"		},
[115] = { 2,	TC,		SEN(getgroups),			"getgroups"		},
[116] = { 2,	TC,		SEN(setgroups),			"setgroups"		},
[117] = { 3,	TC,		SEN(setresuid),			"setresuid"		},
[118] = { 3,	TC,		SEN(getresuid),			"getresuid"		},
[119] = { 3,	TC,		SEN(setresgid),			"setresgid"		},
[120] = { 3,	TC,		SEN(getresgid),			"getresgid"		},
[121] = { 1,	0,		SEN(getpgid),			"getpgid"		},
[122] = { 1,	TC|NF,		SEN(setfsuid),			"setfsuid"		},
[123] = { 1,	TC|NF,		SEN(setfsgid),			"setfsgid"		},
[124] = { 1,	0,		SEN(getsid),			"getsid"		},
[125] = { 2,	TC,		SEN(capget),			"capget"		},
[126] = { 2,	TC,		SEN(capset),			"capset"		},
[127] = { 2,	TS,		SEN(printargs),			"rt_sigpending#64"	},
[128] = { 4,	TS,		SEN(printargs),			"rt_sigtimedwait#64"	},
[129] = { 3,	TS|TP,		SEN(printargs),			"rt_sigqueueinfo#64"	},
[130] = { 2,	TS,		SEN(rt_sigsuspend),		"rt_sigsuspend"		},
[131] = { 2,	TS,		SEN(printargs),			"sigaltstack#64"	},
[132] = { 2,	TF,		SEN(utime),			"utime"			},
[133] = { 3,	TF,		SEN(mknod),			"mknod"			},
[134] = { 1,	TF,		SEN(printargs),			"uselib#64"		},
[135] = { 1,	NF,		SEN(personality),		"personality"		},
[136] = { 2,	TSFA,		SEN(ustat),			"ustat"			},
[137] = { 2,	TF|TSF|TSFA,	SEN(statfs),			"statfs"		},
[138] = { 2,	TD|TFSF|TSFA,	SEN(fstatfs),			"fstatfs"		},
[139] = { 3,	0,		SEN(sysfs),			"sysfs"			},
[140] = { 2,	0,		SEN(getpriority),		"getpriority"		},
[141] = { 3,	0,		SEN(setpriority),		"setpriority"		},
[142] = { 2,	0,		SEN(sched_setparam),		"sched_setparam"	},
[143] = { 2,	0,		SEN(sched_getparam),		"sched_getparam"	},
[144] = { 3,	0,		SEN(sched_setscheduler),	"sched_setscheduler"	},
[145] = { 1,	0,		SEN(sched_getscheduler),	"sched_getscheduler"	},
[146] = { 1,	0,		SEN(sched_get_priority_max),	"sched_get_priority_max"},
[147] = { 1,	0,		SEN(sched_get_priority_min),	"sched_get_priority_min"},
[148] = { 2,	0,		SEN(sched_rr_get_interval_time64),"sched_rr_get_interval"},
[149] = { 2,	TM,		SEN(mlock),			"mlock"			},
[150] = { 2,	TM,		SEN(munlock),			"munlock"		},
[151] = { 1,	TM,		SEN(mlockall),			"mlockall"		},
[152] = { 0,	TM,		SEN(munlockall),		"munlockall"		},
[153] = { 0,	0,		SEN(vhangup),			"vhangup"		},
[154] = { 3,	0,		SEN(modify_ldt),		"modify_ldt"		},
[155] = { 2,	TF,		SEN(pivotroot),			"pivot_root"		},
[156] = { 1,	0,		SEN(printargs),			"_sysctl#64"		},
[157] = { 5,	CC|TC,		SEN(prctl),			"prctl"			},
[158] = { 2,	0,		SEN(arch_prctl),		"arch_prctl"		},
[159] = { 1,	TCL,		SEN(adjtimex64),		"adjtimex"		},
[160] = { 2,	0,		SEN(setrlimit),			"setrlimit"		},
[161] = { 1,	TF,		SEN(chroot),			"chroot"		},
[162] = { 0,	0,		SEN(sync),			"sync"			},
[163] = { 1,	TF,		SEN(acct),			"acct"			},
[164] = { 2,	TCL,		SEN(settimeofday),		"settimeofday"		},
[165] = { 5,	TF,		SEN(mount),			"mount"			},
[166] = { 2,	TF,		SEN(umount2),			"umount2"		},
[167] = { 2,	TF,		SEN(swapon),			"swapon"		},
[168] = { 1,	TF,		SEN(swapoff),			"swapoff"		},
[169] = { 4,	0,		SEN(reboot),			"reboot"		},
[170] = { 2,	0,		SEN(sethostname),		"sethostname"		},
[171] = { 2,	0,		SEN(setdomainname),		"setdomainname"		},
[172] = { 1,	0,		SEN(iopl),			"iopl"			},
[173] = { 3,	0,		SEN(ioperm),			"ioperm"		},
[174] = { 2,	0,		SEN(printargs),			"create_module#64"	},
[175] = { 3,	0,		SEN(init_module),		"init_module"		},
[176] = { 2,	0,		SEN(delete_module),		"delete_module"		},
[177] = { 1,	0,		SEN(printargs),			"get_kernel_syms#64"	},
[178] = { 5,	0,		SEN(printargs),			"query_module#64"	},
[179] = { 4,	TF,		SEN(quotactl),			"quotactl"		},
[180] = { 3,	0,		SEN(printargs),			"nfsservctl#64"		},
[181] = { 5,	TN,		SEN(getpmsg),			"getpmsg"		},
[182] = { 5,	TN,		SEN(putpmsg),			"putpmsg"		},
[183] = { 5,	0,		SEN(afs_syscall),		"afs_syscall"		},
[184] = { 3,	0,		SEN(tuxcall),			"tuxcall"		},
[185] = { 3,	0,		SEN(security),			"security"		},
[186] = { 0,	PU|NF,		SEN(gettid),			"gettid"		},
[187] = { 3,	TD,		SEN(readahead),			"readahead"		},
[188] = { 5,	TF,		SEN(setxattr),			"setxattr"		},
[189] = { 5,	TF,		SEN(setxattr),			"lsetxattr"		},
[190] = { 5,	TD,		SEN(fsetxattr),			"fsetxattr"		},
[191] = { 4,	TF,		SEN(getxattr),			"getxattr"		},
[192] = { 4,	TF,		SEN(getxattr),			"lgetxattr"		},
[193] = { 4,	TD,		SEN(fgetxattr),			"fgetxattr"		},
[194] = { 3,	TF,		SEN(listxattr),			"listxattr"		},
[195] = { 3,	TF,		SEN(listxattr),			"llistxattr"		},
[196] = { 3,	TD,		SEN(flistxattr),		"flistxattr"		},
[197] = { 2,	TF,		SEN(removexattr),		"removexattr"		},
[198] = { 2,	TF,		SEN(removexattr),		"lremovexattr"		},
[199] = { 2,	TD,		SEN(fremovexattr),		"fremovexattr"		},
[200] = { 2,	TS|TP,		SEN(tkill),			"tkill"			},
[201] = { 1,	TCL,		SEN(time),			"time"			},
[202] = { 6,	0,		SEN(futex_time64),		"futex"			},
[203] = { 3,	0,		SEN(sched_setaffinity),		"sched_setaffinity"	},
[204] = { 3,	0,		SEN(sched_getaffinity),		"sched_getaffinity"	},
[205] = { 1,	0,		SEN(printargs),			"set_thread_area#64"	},
[206] = { 2,	TM,		SEN(printargs),			"io_setup#64"		},
[207] = { 1,	TM,		SEN(io_destroy),		"io_destroy"		},
[208] = { 5,	0,		SEN(io_getevents_time64),	"io_getevents"		},
[209] = { 3,	0,		SEN(printargs),			"io_submit#64"		},
[210] = { 3,	0,		SEN(io_cancel),			"io_cancel"		},
[211] = { 1,	0,		SEN(printargs),			"get_thread_area#64"	},
[212] = { 3,	0,		SEN(lookup_dcookie),		"lookup_dcookie"	},
[213] = { 1,	TD,		SEN(epoll_create),		"epoll_create"		},
[214] = { 4,	0,		SEN(printargs),			"epoll_ctl_old#64"	},
[215] = { 4,	0,		SEN(printargs),			"epoll_wait_old#64"	},
[216] = { 5,	TM|SI,		SEN(remap_file_pages),		"remap_file_pages"	},
[217] = { 3,	TD,		SEN(getdents64),		"getdents64"		},
[218] = { 1,	0,		SEN(set_tid_address),		"set_tid_address"	},
[219] = { 0,	0,		SEN(restart_syscall),		"restart_syscall"	},
[220] = { 4,	TI,		SEN(semtimedop_time64),		"semtimedop"		},
[221] = { 4,	TD,		SEN(fadvise64),			"fadvise64"		},
[222] = { 3,	0,		SEN(printargs),			"timer_create#64"	},
[223] = { 4,	0,		SEN(timer_settime64),		"timer_settime"		},
[224] = { 2,	0,		SEN(timer_gettime64),		"timer_gettime"		},
[225] = { 1,	0,		SEN(timer_getoverrun),		"timer_getoverrun"	},
[226] = { 1,	0,		SEN(timer_delete),		"timer_delete"		},
[227] = { 2,	TCL,		SEN(clock_settime64),		"clock_settime"		},
[228] = { 2,	TCL,		SEN(clock_gettime64),		"clock_gettime"		},
[229] = { 2,	TCL,		SEN(clock_getres_time64),	"clock_getres"		},
[230] = { 4,	0,		SEN(clock_nanosleep_time64),	"clock_nanosleep"	},
[231] = { 1,	TP|SE,		SEN(exit),			"exit_group"		},
[232] = { 4,	TD,		SEN(epoll_wait),		"epoll_wait"		},
[233] = { 4,	TD,		SEN(epoll_ctl),			"epoll_ctl"		},
[234] = { 3,	TS|TP,		SEN(tgkill),			"tgkill"		},
[235] = { 2,	TF,		SEN(utimes),			"utimes"		},
[236] = { 5,	0,		SEN(printargs),			"vserver#64"		},
[237] = { 6,	TM,		SEN(mbind),			"mbind"			},
[238] = { 3,	TM,		SEN(set_mempolicy),		"set_mempolicy"		},
[239] = { 5,	TM,		SEN(get_mempolicy),		"get_mempolicy"		},
[240] = { 4,	TD,		SEN(mq_open),			"mq_open"		},
[241] = { 1,	0,		SEN(mq_unlink),			"mq_unlink"		},
[242] = { 5,	TD,		SEN(mq_timedsend_time64),	"mq_timedsend"		},
[243] = { 5,	TD,		SEN(mq_timedreceive_time64),	"mq_timedreceive"	},
[244] = { 2,	0,		SEN(printargs),			"mq_notify#64"		},
[245] = { 3,	TD,		SEN(mq_getsetattr),		"mq_getsetattr"		},
[246] = { 4,	0,		SEN(printargs),			"kexec_load#64"		},
[247] = { 5,	TP,		SEN(printargs),			"waitid#64"		},
[248] = { 5,	0,		SEN(add_key),			"add_key"		},
[249] = { 4,	0,		SEN(request_key),		"request_key"		},
[250] = { 5,	0,		SEN(keyctl),			"keyctl"		},
[251] = { 3,	0,		SEN(ioprio_set),		"ioprio_set"		},
[252] = { 2,	0,		SEN(ioprio_get),		"ioprio_get"		},
[253] = { 0,	TD,		SEN(inotify_init),		"inotify_init"		},
[254] = { 3,	TD|TF,		SEN(inotify_add_watch),		"inotify_add_watch"	},
[255] = { 2,	TD,		SEN(inotify_rm_watch),		"inotify_rm_watch"	},
[256] = { 4,	TM,		SEN(migrate_pages),		"migrate_pages"		},
[257] = { 4,	TD|TF,		SEN(openat),			"openat"		},
[258] = { 3,	TD|TF,		SEN(mkdirat),			"mkdirat"		},
[259] = { 4,	TD|TF,		SEN(mknodat),			"mknodat"		},
[260] = { 5,	TD|TF,		SEN(fchownat),			"fchownat"		},
[261] = { 3,	TD|TF,		SEN(futimesat),			"futimesat"		},
[262] = { 4,	TD|TF|TFST|TSTA,SEN(newfstatat),		"newfstatat"		},
[263] = { 3,	TD|TF,		SEN(unlinkat),			"unlinkat"		},
[264] = { 4,	TD|TF,		SEN(renameat),			"renameat"		},
[265] = { 5,	TD|TF,		SEN(linkat),			"linkat"		},
[266] = { 3,	TD|TF,		SEN(symlinkat),			"symlinkat"		},
[267] = { 4,	TD|TF,		SEN(readlinkat),		"readlinkat"		},
[268] = { 3,	TD|TF,		SEN(fchmodat),			"fchmodat"		},
[269] = { 3,	TD|TF,		SEN(faccessat),			"faccessat"		},
[270] = { 6,	TD,		SEN(pselect6_time64),		"pselect6"		},
[271] = { 5,	TD,		SEN(ppoll_time64),		"ppoll"			},
[272] = { 1,	0,		SEN(unshare),			"unshare"		},
[273] = { 2,	0,		SEN(printargs),			"set_robust_list#64"	},
[274] = { 3,	0,		SEN(printargs),			"get_robust_list#64"	},
[275] = { 6,	TD,		SEN(splice),			"splice"		},
[276] = { 4,	TD,		SEN(tee),			"tee"			},
[277] = { 4,	TD,		SEN(sync_file_range),		"sync_file_range"	},
[278] = { 4,	TD,		SEN(printargs),			"vmsplice#64"		},
[279] = { 6,	TM,		SEN(printargs),			"move_pages#64"		},
[280] = { 4,	TD|TF,		SEN(utimensat_time64),		"utimensat"		},
[281] = { 6,	TD,		SEN(epoll_pwait),		"epoll_pwait"		},
[282] = { 3,	TD|TS,		SEN(signalfd),			"signalfd"		},
[283] = { 2,	TD,		SEN(timerfd_create),		"timerfd_create"	},
[284] = { 1,	TD,		SEN(eventfd),			"eventfd"		},
[285] = { 4,	TD,		SEN(fallocate),			"fallocate"		},
[286] = { 4,	TD,		SEN(timerfd_settime64),		"timerfd_settime"	},
[287] = { 2,	TD,		SEN(timerfd_gettime64),		"timerfd_gettime"	},
[288] = { 4,	TN,		SEN(accept4),			"accept4"		},
[289] = { 4,	TD|TS,		SEN(signalfd4),			"signalfd4"		},
[290] = { 2,	TD,		SEN(eventfd2),			"eventfd2"		},
[291] = { 1,	TD,		SEN(epoll_create1),		"epoll_create1"		},
[292] = { 3,	TD,		SEN(dup3),			"dup3"			},
[293] = { 2,	TD,		SEN(pipe2),			"pipe2"			},
[294] = { 1,	TD,		SEN(inotify_init1),		"inotify_init1"		},
[295] = { 4,	TD,		SEN(printargs),			"preadv#64"		},
[296] = { 4,	TD,		SEN(printargs),			"pwritev#64"		},
[297] = { 4,	TP|TS,		SEN(printargs),			"rt_tgsigqueueinfo#64"	},
[298] = { 5,	TD,		SEN(perf_event_open),		"perf_event_open"	},
[299] = { 5,	TN,		SEN(printargs),			"recvmmsg#64"		},
[300] = { 2,	TD,		SEN(fanotify_init),		"fanotify_init"		},
[301] = { 5,	TD|TF,		SEN(fanotify_mark),		"fanotify_mark"		},
[302] = { 4,	0,		SEN(prlimit64),			"prlimit64"		},
[303] = { 5,	TD|TF,		SEN(name_to_handle_at),		"name_to_handle_at"	},
[304] = { 3,	TD,		SEN(open_by_handle_at),		"open_by_handle_at"	},
[305] = { 2,	TCL,		SEN(clock_adjtime64),		"clock_adjtime"		},
[306] = { 1,	TD,		SEN(syncfs),			"syncfs"		},
[307] = { 4,	TN,		SEN(printargs),			"sendmmsg#64"		},
[308] = { 2,	TD,		SEN(setns),			"setns"			},
[309] = { 3,	0,		SEN(getcpu),			"getcpu"		},
[310] = { 6,	0,		SEN(printargs),			"process_vm_readv#64"	},
[311] = { 6,	0,		SEN(printargs),			"process_vm_writev#64"	},
[312] = { 5,	0,		SEN(kcmp),			"kcmp"			},
[313] = { 3,	TD,		SEN(finit_module),		"finit_module"		},
[314] = { 3,	0,		SEN(sched_setattr),		"sched_setattr"		},
[315] = { 4,	0,		SEN(sched_getattr),		"sched_getattr"		},
[316] = { 5,	TD|TF,		SEN(renameat2),			"renameat2"		},
[317] = { 3,	0,		SEN(seccomp),			"seccomp"		},
[318] = { 3,	0,		SEN(getrandom),			"getrandom"		},
[319] = { 2,	TD,		SEN(memfd_create),		"memfd_create"		},
[320] = { 5,	TD,		SEN(kexec_file_load),		"kexec_file_load"	},
[321] = { 3,	TD,		SEN(bpf),			"bpf"			},
[322] = { 5,	CC|TD|TF|TP|TSD|SE|SI,	SEN(printargs),			"execveat#64"		},
[323] = { 1,	TD,		SEN(userfaultfd),		"userfaultfd"		},
[324] = { 3,	0,		SEN(membarrier),		"membarrier"		},
[325] = { 3,	TM,		SEN(mlock2),			"mlock2"		},
[326] = { 6,	TD,		SEN(copy_file_range),		"copy_file_range"	},
[327] = { 6,	TD,		SEN(printargs),			"preadv2#64"		},
[328] = { 6,	TD,		SEN(printargs),			"pwritev2#64"		},
[329] = { 4,	TM|SI,		SEN(pkey_mprotect),		"pkey_mprotect"		},
[330] = { 2,	0,		SEN(pkey_alloc),		"pkey_alloc"		},
[331] = { 1,	0,		SEN(pkey_free),			"pkey_free"		},
[332] = { 5,	TD|TF|TFST|TSTA,SEN(statx),			"statx"			},
[333] = { 6,	0,		SEN(io_pgetevents_time64),	"io_pgetevents"		},
[334] = { 4,	0,		SEN(rseq),			"rseq"			},
/* [335 ... 423] - reserved to sync up with other architectures */
#include "syscallent-common.h"
[453] = { 3,	TM,		SEN(map_shadow_stack),		"map_shadow_stack"	},
/*
 * x32-specific system call numbers start at 512 to avoid cache impact
 * for native 64-bit operation.
 */
[512] = { 4,	CST|TS,		SEN(rt_sigaction),		"rt_sigaction"		},
[513] = { 0,	CST|TS,		SEN(rt_sigreturn),		"rt_sigreturn"		},
[514] = { 3,	CST|TD,		SEN(ioctl),			"ioctl"			},
[515] = { 3,	CST|TD,		SEN(readv),			"readv"			},
[516] = { 3,	CST|TD,		SEN(writev),			"writev"		},
[517] = { 6,	CST|TN,		SEN(recvfrom),			"recvfrom"		},
[518] = { 3,	CST|TN,		SEN(sendmsg),			"sendmsg"		},
[519] = { 3,	CST|TN,		SEN(recvmsg),			"recvmsg"		},
[520] = { 3,	CST|CC|TF|TP|TSD|SE|SI,SEN(execve),			"execve"		},
[521] = { 4,	CST,		SEN(ptrace),			"ptrace"		},
[522] = { 2,	CST|TS,		SEN(rt_sigpending),		"rt_sigpending"		},
[523] = { 4,	CST|TS,		SEN(rt_sigtimedwait_time64),	"rt_sigtimedwait"	},
[524] = { 3,	CST|TS|TP,		SEN(rt_sigqueueinfo),		"rt_sigqueueinfo"	},
[525] = { 2,	CST|TS,		SEN(sigaltstack),		"sigaltstack"		},
[526] = { 3,	CST,		SEN(timer_create),		"timer_create"		},
[527] = { 2,	CST|TD,		SEN(mq_notify),			"mq_notify"		},
[528] = { 4,	CST,		SEN(kexec_load),		"kexec_load"		},
[529] = { 5,	CST|TP,		SEN(waitid),			"waitid"		},
[530] = { 2,	CST,		SEN(set_robust_list),		"set_robust_list"	},
[531] = { 3,	CST,		SEN(get_robust_list),		"get_robust_list"	},
[532] = { 4,	CST|TD,		SEN(vmsplice),			"vmsplice"		},
[533] = { 6,	CST|TM,		SEN(move_pages),		"move_pages"		},
[534] = { 4,	TD,		SEN(preadv),			"preadv"		},
[535] = { 4,	TD,		SEN(pwritev),			"pwritev"		},
[536] = { 4,	CST|TP|TS,	SEN(rt_tgsigqueueinfo),		"rt_tgsigqueueinfo"	},
[537] = { 5,	CST|TN,		SEN(recvmmsg_time64),		"recvmmsg"		},
[538] = { 4,	CST|TN,		SEN(sendmmsg),			"sendmmsg"		},
[539] = { 6,	CST,		SEN(process_vm_readv),		"process_vm_readv"	},
[540] = { 6,	CST,		SEN(process_vm_writev),		"process_vm_writev"	},
[541] = { 5,	CST|TN,		SEN(setsockopt),		"setsockopt"		},
[542] = { 5,	CST|TN,		SEN(getsockopt),		"getsockopt"		},
[543] = { 2,	CST|TM,		SEN(io_setup),			"io_setup"		},
[544] = { 3,	CST,		SEN(io_submit),			"io_submit"		},
[545] = { 5,	CST|CC|TD|TF|TP|TSD|SE|SI,	SEN(execveat),		"execveat"		},
[546] = { 5,	TD,		SEN(preadv2),			"preadv2"		},
[547] = { 5,	TD,		SEN(pwritev2),			"pwritev2"		},
