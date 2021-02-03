(function() {var implementors = {};
implementors["base64"] = [{"text":"impl&lt;'a, R:&nbsp;Read&gt; Read for DecoderReader&lt;'a, R&gt;","synthetic":false,"types":[]}];
implementors["brotli"] = [{"text":"impl&lt;R:&nbsp;Read, BufferType:&nbsp;SliceWrapperMut&lt;u8&gt;, Alloc:&nbsp;BrotliAlloc&gt; Read for CompressorReaderCustomAlloc&lt;R, BufferType, Alloc&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for CompressorReader&lt;R&gt;","synthetic":false,"types":[]}];
implementors["brotli_decompressor"] = [{"text":"impl&lt;R:&nbsp;Read, BufferType:&nbsp;SliceWrapperMut&lt;u8&gt;, AllocU8:&nbsp;Allocator&lt;u8&gt;, AllocU32:&nbsp;Allocator&lt;u32&gt;, AllocHC:&nbsp;Allocator&lt;HuffmanCode&gt;&gt; Read for DecompressorCustomAlloc&lt;R, BufferType, AllocU8, AllocU32, AllocHC&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for Decompressor&lt;R&gt;","synthetic":false,"types":[]}];
implementors["flate2"] = [{"text":"impl&lt;R:&nbsp;Read&gt; Read for CrcReader&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for DeflateEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for DeflateDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for DeflateEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for DeflateDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;W:&nbsp;Read + Write&gt; Read for DeflateEncoder&lt;W&gt;","synthetic":false,"types":[]},{"text":"impl&lt;W:&nbsp;Read + Write&gt; Read for DeflateDecoder&lt;W&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for GzEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for GzDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for MultiGzDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for GzEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for GzDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for MultiGzDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read + Write&gt; Read for GzEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;W:&nbsp;Read + Write&gt; Read for GzDecoder&lt;W&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for ZlibEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for ZlibDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for ZlibEncoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for ZlibDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;W:&nbsp;Read + Write&gt; Read for ZlibEncoder&lt;W&gt;","synthetic":false,"types":[]},{"text":"impl&lt;W:&nbsp;Read + Write&gt; Read for ZlibDecoder&lt;W&gt;","synthetic":false,"types":[]}];
implementors["lz4"] = [{"text":"impl&lt;R:&nbsp;Read&gt; Read for Decoder&lt;R&gt;","synthetic":false,"types":[]}];
implementors["mio"] = [{"text":"impl Read for Receiver","synthetic":false,"types":[]},{"text":"impl Read for &amp;Receiver","synthetic":false,"types":[]}];
implementors["parquet"] = [{"text":"impl&lt;R:&nbsp;ParquetReader&gt; Read for FileSource&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl Read for SliceableCursor","synthetic":false,"types":[]}];
implementors["rand_core"] = [{"text":"impl Read for dyn RngCore","synthetic":false,"types":[]}];
implementors["snap"] = [{"text":"impl&lt;R:&nbsp;Read&gt; Read for FrameDecoder&lt;R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;Read&gt; Read for FrameEncoder&lt;R&gt;","synthetic":false,"types":[]}];
implementors["thrift"] = [{"text":"impl&lt;C&gt; Read for TBufferedReadTransport&lt;C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Read,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl&lt;C&gt; Read for TFramedReadTransport&lt;C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Read,&nbsp;</span>","synthetic":false,"types":[]},{"text":"impl Read for TBufferChannel","synthetic":false,"types":[]},{"text":"impl Read for TTcpChannel","synthetic":false,"types":[]},{"text":"impl&lt;C&gt; Read for ReadHalf&lt;C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;C: Read,&nbsp;</span>","synthetic":false,"types":[]}];
implementors["zstd"] = [{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for Decoder&lt;'_, R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R:&nbsp;BufRead&gt; Read for Encoder&lt;'_, R&gt;","synthetic":false,"types":[]},{"text":"impl&lt;R, D&gt; Read for Reader&lt;R, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;R: BufRead,<br>&nbsp;&nbsp;&nbsp;&nbsp;D: Operation,&nbsp;</span>","synthetic":false,"types":[]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()