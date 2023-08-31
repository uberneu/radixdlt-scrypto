(function() {var implementors = {
"radix_engine":[["impl&lt;'a, E, F: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.FnMut.html\" title=\"trait radix_engine::types::prelude::FnMut\">FnMut</a>(<a class=\"enum\" href=\"radix_engine/track/interface/enum.IOAccess.html\" title=\"enum radix_engine::track::interface::IOAccess\">IOAccess</a>) -&gt; <a class=\"enum\" href=\"radix_engine/types/prelude/enum.Result.html\" title=\"enum radix_engine::types::prelude::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.70.0/std/primitive.unit.html\">()</a>, E&gt;, M: <a class=\"trait\" href=\"radix_engine_store_interface/db_key_mapper/trait.DatabaseKeyMapper.html\" title=\"trait radix_engine_store_interface::db_key_mapper::DatabaseKeyMapper\">DatabaseKeyMapper</a> + 'static, K: <a class=\"trait\" href=\"radix_engine_store_interface/db_key_mapper/trait.SubstateKeyContent.html\" title=\"trait radix_engine_store_interface::db_key_mapper::SubstateKeyContent\">SubstateKeyContent</a> + 'static&gt; <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine/track/track/struct.TracedIterator.html\" title=\"struct radix_engine::track::track::TracedIterator\">TracedIterator</a>&lt;'a, E, F, M, K&gt;"],["impl&lt;'a, E&gt; <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine/track/state_updates/struct.IterationCountedIter.html\" title=\"struct radix_engine::track::state_updates::IterationCountedIter\">IterationCountedIter</a>&lt;'a, E&gt;"],["impl&lt;K, V, U, O&gt; <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine/track/utils/struct.OverlayingIterator.html\" title=\"struct radix_engine::track::utils::OverlayingIterator\">OverlayingIterator</a>&lt;U, O&gt;<span class=\"where fmt-newline\">where\n    K: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Ord.html\" title=\"trait radix_engine::types::prelude::Ord\">Ord</a>,\n    U: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a>&lt;Item = <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.70.0/std/primitive.tuple.html\">(K, V)</a>&gt;,\n    O: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a>&lt;Item = (K, <a class=\"enum\" href=\"radix_engine/types/prelude/enum.Option.html\" title=\"enum radix_engine::types::prelude::Option\">Option</a>&lt;V&gt;)&gt;,</span>"],["impl&lt;K, V, U, O, E&gt; <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine/track/utils/struct.OverlayingResultIterator.html\" title=\"struct radix_engine::track::utils::OverlayingResultIterator\">OverlayingResultIterator</a>&lt;U, O&gt;<span class=\"where fmt-newline\">where\n    K: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Ord.html\" title=\"trait radix_engine::types::prelude::Ord\">Ord</a>,\n    U: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a>&lt;Item = <a class=\"enum\" href=\"radix_engine/types/prelude/enum.Result.html\" title=\"enum radix_engine::types::prelude::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/1.70.0/std/primitive.tuple.html\">(K, V)</a>, E&gt;&gt;,\n    O: <a class=\"trait\" href=\"radix_engine/types/prelude/trait.Iterator.html\" title=\"trait radix_engine::types::prelude::Iterator\">Iterator</a>&lt;Item = (K, <a class=\"enum\" href=\"radix_engine/types/prelude/enum.Option.html\" title=\"enum radix_engine::types::prelude::Option\">Option</a>&lt;V&gt;)&gt;,</span>"]],
"radix_engine_common":[],
"radix_engine_interface":[["impl <a class=\"trait\" href=\"radix_engine_interface/prelude/prelude/trait.Iterator.html\" title=\"trait radix_engine_interface::prelude::prelude::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine_interface/api/object_api/struct.ModuleIdIter.html\" title=\"struct radix_engine_interface::api::object_api::ModuleIdIter\">ModuleIdIter</a>"],["impl <a class=\"trait\" href=\"radix_engine_interface/prelude/prelude/trait.Iterator.html\" title=\"trait radix_engine_interface::prelude::prelude::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine_interface/api/object_api/struct.AttachedModuleIdIter.html\" title=\"struct radix_engine_interface::api::object_api::AttachedModuleIdIter\">AttachedModuleIdIter</a>"]],
"radix_engine_stores":[["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.70.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine_stores/hash_tree/types/struct.NibbleIterator.html\" title=\"struct radix_engine_stores::hash_tree::types::NibbleIterator\">NibbleIterator</a>&lt;'a&gt;"],["impl&lt;'a, P&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.70.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine_stores/hash_tree/jellyfish/struct.NibbleRangeIterator.html\" title=\"struct radix_engine_stores::hash_tree::jellyfish::NibbleRangeIterator\">NibbleRangeIterator</a>&lt;'a, P&gt;"],["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.70.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine_stores/hash_tree/types/struct.LeafKeyBitIterator.html\" title=\"struct radix_engine_stores::hash_tree::types::LeafKeyBitIterator\">LeafKeyBitIterator</a>&lt;'a&gt;"],["impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.70.0/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"radix_engine_stores/hash_tree/types/struct.BitIterator.html\" title=\"struct radix_engine_stores::hash_tree::types::BitIterator\">BitIterator</a>&lt;'a&gt;"]],
"sbor":[],
"scrypto":[],
"scrypto_test":[]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()